use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::BufMut;
use prost::Message;
use prost_reflect::MethodDescriptor;
use prost_reflect::{DescriptorPool, DynamicMessage};
use serde_json::Value as JsonValue;
use std::sync::OnceLock;

use crate::metrics::metrics::Metrics;
use crate::requester::error::RequestError;
use crate::requester::Requester;
use tonic::client::Grpc;
use tonic::transport::Channel;

use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::Status;

pub struct GrpcRequester<'a> {
    metrics: &'a Arc<Metrics>,
    address: String,
    proto: String,
    method: String,
    data: Option<String>,
    timeout: u64,

    path_uri: OnceLock<http::Uri>,
    codec: OnceLock<DynamicCodec>,
    req_msg: OnceLock<DynamicMessage>,
    client: OnceLock<Grpc<Channel>>,
    channel: OnceLock<Channel>,
}

impl<'a> GrpcRequester<'a> {
    pub fn new(
        metrics: &'a Arc<Metrics>,
        address: String,
        proto: String,
        method: String,
        data: Option<String>,
        timeout: u64,
    ) -> Self {
        Self {
            metrics,
            address,
            proto,
            method,
            data,
            timeout,
            path_uri: OnceLock::new(),
            codec: OnceLock::new(),
            req_msg: OnceLock::new(),
            client: OnceLock::new(),
            channel: OnceLock::new(),
        }
    }
}

impl<'a> Requester for GrpcRequester<'a> {
    async fn initialize(&self) -> Result<(), crate::requester::error::RequestError> {
        if self.client.get().is_some() {
            return Ok(());
        }

        let pool = load_proto(&self.proto).map_err(|e| {
            RequestError::ConfigError(format!("Failed to load proto '{}': {}", self.proto, e))
        })?;

        let method = get_method(&pool, &self.method)
            .map_err(|e| RequestError::ConfigError(format!("Failed to get method: {}", e)))?;

        let req_msg_val = if let Some(json_data) = &self.data {
            build_request(&method, json_data).map_err(|e| {
                RequestError::InvalidRequest(format!("Failed to build request: {}", e))
            })?
        } else {
            DynamicMessage::new(method.input())
        };

        let endpoint = tonic::transport::Endpoint::from_shared(self.address.to_string())
            .map_err(|e| RequestError::ConnectionError(format!("Invalid URI: {}", e)))?;

        let channel = endpoint
            .timeout(Duration::from_secs(self.timeout))
            .connect()
            .await
            .map_err(|e| RequestError::ConnectionError(format!("Failed to connect: {}", e)))?;

        let path = format!("/{}/{}", method.parent_service().full_name(), method.name());

        let path_uri_val: http::Uri = path
            .parse()
            .map_err(|e| RequestError::InvalidRequest(format!("Invalid path: {}", e)))?;

        let mut client_val = Grpc::new(channel.clone());

        client_val
            .ready()
            .await
            .map_err(|e| RequestError::ConnectionError(format!("Client not ready: {}", e)))?;

        let codec_val = DynamicCodec::new(method.input().clone(), method.output().clone());

        if self.channel.set(channel).is_err() {
            return Err(RequestError::InternalError(
                "Channel already set".to_string(),
            ));
        }

        if self.client.set(client_val).is_err() {
            return Err(RequestError::InternalError(
                "Client already set".to_string(),
            ));
        }

        if self.path_uri.set(path_uri_val).is_err() {
            return Err(RequestError::InternalError(
                "Path URI already set".to_string(),
            ));
        }

        if self.codec.set(codec_val).is_err() {
            return Err(RequestError::InternalError("Codec already set".to_string()));
        }

        if self.req_msg.set(req_msg_val).is_err() {
            return Err(RequestError::InternalError(
                "Request message already set".to_string(),
            ));
        }

        Ok(())
    }
    async fn request(&self) -> Result<(), RequestError> {
        let stored_client = self.client.get().ok_or(RequestError::InternalError(
            "Requester not initialised: Missing client".to_string(),
        ))?;
        let mut client = stored_client.clone();
        let path_uri_val = self.path_uri.get().ok_or(RequestError::InternalError(
            "Requester not initialised: Missing path_uri".to_string(),
        ))?;
        let codec = self.codec.get().ok_or(RequestError::InternalError(
            "Requester not initialised: Missing codec".to_string(),
        ))?;
        let req_msg = self.req_msg.get().ok_or(RequestError::InternalError(
            "Requester not initialised: Missing request message".to_string(),
        ))?;

        let start = std::time::Instant::now();

        let mut request = tonic::Request::new(req_msg.clone());

        request.metadata_mut().insert(
            "te",
            tonic::metadata::MetadataValue::try_from("trailers").unwrap(),
        );

        let path_and_query = path_uri_val
            .path_and_query()
            .ok_or(RequestError::InternalError(
                "Path URI is missing path and query component.".to_string(),
            ))?
            .clone();

        client.ready().await.map_err(|e| {
            RequestError::ConnectionError(format!("Client not ready (in request): {}", e))
        })?;

        client
            .unary(request, path_and_query, codec.clone())
            .await
            .map_err(|e| {
                println!("gRPC error details: {:?}", e);
                RequestError::GrpcError(format!("gRPC call failed: {}", e))
            })?;

        let req_duration = start.elapsed();

        self.metrics
            .record_latency(req_duration.as_micros().try_into().unwrap_or(0))
            .await;

        Ok(())
    }
}

fn load_proto(path: &str) -> Result<DescriptorPool, Box<dyn std::error::Error>> {
    // Check if it's a compiled descriptor or source file
    if path.ends_with(".proto") {
        // Compile the .proto file on the fly
        let mut pool = DescriptorPool::new();

        // Get the directory containing the .proto file
        let proto_path = std::path::Path::new(path);
        let parent_dir = proto_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine parent directory"))?;

        // Use protox to parse the .proto file
        let file_descriptor_set = protox::compile([path], [parent_dir])?;

        for file in file_descriptor_set.file {
            pool.add_file_descriptor_proto(file)?;
        }

        Ok(pool)
    } else {
        // Assume it's a compiled descriptor set
        let bytes = std::fs::read(path)?;
        let file_descriptor_set = prost_types::FileDescriptorSet::decode(&bytes[..])?;

        let mut pool = DescriptorPool::new();
        for file in file_descriptor_set.file {
            pool.add_file_descriptor_proto(file)?;
        }

        Ok(pool)
    }
}

fn get_method<'a>(
    pool: &'a DescriptorPool,
    full_method: &str,
) -> anyhow::Result<prost_reflect::MethodDescriptor> {
    let (service_name, method_name) = if let Some((svc, method)) = full_method.rsplit_once('/') {
        (svc.trim(), method.trim())
    } else if let Some((svc, method)) = full_method.rsplit_once('.') {
        (svc.trim(), method.trim())
    } else {
        return Err(anyhow::anyhow!(
            "invalid format: expected Service.Method or Service/Method, got: '{}'",
            full_method
        ));
    };

    let service = pool
        .get_service_by_name(service_name)
        .ok_or_else(|| anyhow::anyhow!("service not found: {}", service_name))?;

    let methods: Vec<_> = service.methods().collect();
    let method = methods
        .into_iter()
        .find(|m| m.name() == method_name)
        .ok_or_else(|| anyhow::anyhow!("method not found: {}", method_name))?;

    Ok(method)
}

fn build_request(method: &MethodDescriptor, json: &str) -> anyhow::Result<DynamicMessage> {
    let json_value: JsonValue = serde_json::from_str(json)?;

    let msg = DynamicMessage::deserialize(method.input().clone(), &json_value)?;

    Ok(msg)
}

// Custom codec for dynamic messages
#[derive(Debug, Clone)]
struct DynamicCodec {
    input_desc: prost_reflect::MessageDescriptor,
    output_desc: prost_reflect::MessageDescriptor,
}

impl DynamicCodec {
    fn new(
        input_desc: prost_reflect::MessageDescriptor,
        output_desc: prost_reflect::MessageDescriptor,
    ) -> Self {
        Self {
            input_desc,
            output_desc,
        }
    }
}

impl Codec for DynamicCodec {
    type Encode = DynamicMessage;
    type Decode = DynamicMessage;
    type Encoder = DynamicEncoder;
    type Decoder = DynamicDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        DynamicEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        DynamicDecoder {
            output_desc: self.output_desc.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct DynamicEncoder;

impl Encoder for DynamicEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        let bytes = item.encode_to_vec();
        buf.put_slice(&bytes);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct DynamicDecoder {
    output_desc: prost_reflect::MessageDescriptor,
}

impl Decoder for DynamicDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        use bytes::Buf;

        let remaining = buf.remaining();

        if remaining == 0 {
            // If the buffer is empty, it means the server sent a successful, zero-length message
            // We must decode this as an empty message and return it as the result.
            let empty_msg = DynamicMessage::new(self.output_desc.clone());
            return Ok(Some(empty_msg));
        }

        // Proceed with decoding the data.
        let mut bytes = vec![0u8; remaining];
        buf.copy_to_slice(&mut bytes);

        // Decode the dynamic message
        let msg = DynamicMessage::decode(self.output_desc.clone(), &bytes[..])
            .map_err(|e| Status::internal(format!("Failed to decode response: {}", e)))?;

        Ok(Some(msg))
    }
}
