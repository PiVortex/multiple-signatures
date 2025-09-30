use crate::*;
use ::mpc_contract::{
    crypto_shared::{ed25519_types, k256_types, SignatureResponse},
    primitives::signature::{SignRequest, SignRequestArgs},
};
use k256::elliptic_curve::group::GroupEncoding;

#[near(serializers = [borsh])]
#[derive(Clone)]
pub enum StoredSignatureResponse {
    Secp256k1 {
        big_r_bytes: [u8; 33],
        s: k256_types::SerializableScalar,
        recovery_id: u8,
    },
    Ed25519 {
        signature: ed25519_types::Signature,
    },
}

#[near(serializers = [borsh])]
#[derive(Clone)]
pub struct StoredRequestAndResponse {
    pub request: SignRequest,
    pub response: Result<StoredSignatureResponse, ()>,
}

#[near(serializers = [json])]
pub struct RequestAndResponse {
    pub request: SignRequestArgs,
    pub response: Result<SignatureResponse, ()>,
}

impl StoredSignatureResponse {
    /// Convert from MPC contract's SignatureResponse to our stored format
    pub fn from_signature_response(response: SignatureResponse) -> Result<Self, ()> {
        let signature_data = match response {
            SignatureResponse::Secp256k1(sig) => {
                let big_r_compressed: [u8; 33] = sig.big_r.affine_point.to_bytes().into();

                StoredSignatureResponse::Secp256k1 {
                    big_r_bytes: big_r_compressed,
                    s: sig.s,
                    recovery_id: sig.recovery_id,
                }
            }
            SignatureResponse::Ed25519 { signature } => {
                StoredSignatureResponse::Ed25519 { signature }
            }
        };

        Ok(signature_data)
    }

    /// Create an error response
    pub fn from_error() -> Result<Self, ()> {
        Err(())
    }

    /// Convert back to MPC contract's SignatureResponse format
    pub fn to_signature_response(self) -> Result<SignatureResponse, ()> {
        let mpc_response = match self {
            StoredSignatureResponse::Secp256k1 {
                big_r_bytes,
                s,
                recovery_id,
            } => {
                let big_r = k256::AffinePoint::from_bytes(&big_r_bytes.into())
                    .into_option()
                    .ok_or(())?;

                let serializable_big_r = k256_types::SerializableAffinePoint {
                    affine_point: big_r,
                };

                let signature = k256_types::Signature {
                    big_r: serializable_big_r,
                    s,
                    recovery_id,
                };

                SignatureResponse::Secp256k1(signature)
            }
            StoredSignatureResponse::Ed25519 { signature } => {
                SignatureResponse::Ed25519 { signature }
            }
        };
        Ok(mpc_response)
    }
}

impl RequestAndResponse {
    /// Convert from stored format to API format
    pub fn from_stored(stored: StoredRequestAndResponse) -> Self {
        let request = sign_request_to_sign_request_args(stored.request);
        let response = match stored.response {
            Ok(stored_sig) => stored_sig.to_signature_response(),
            Err(error) => Err(error),
        };

        RequestAndResponse { request, response }
    }
}

/// Convert SignRequestArgs to SignRequest
pub fn sign_request_args_to_sign_request(args: SignRequestArgs) -> SignRequest {
    args.try_into()
        .expect("Failed to convert SignRequestArgs to SignRequest")
}

/// Convert SignRequest back to SignRequestArgs (for external API compatibility)
pub fn sign_request_to_sign_request_args(request: SignRequest) -> SignRequestArgs {
    SignRequestArgs {
        path: request.path,
        payload_v2: Some(request.payload),
        deprecated_payload: None,
        domain_id: Some(request.domain_id),
        deprecated_key_version: None,
    }
}
