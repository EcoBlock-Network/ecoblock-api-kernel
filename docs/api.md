# API overview

Endpoints (key ones):

- POST /tangle — create block (payload: parents[], data JSON, signature base64, public_key)
- GET /tangle — list blocks (pagination)
- GET /tangle/:id — fetch block by UUID
- PUT /tangle/:id — update
- DELETE /tangle/:id — delete

- POST /auth/login — returns JWT
- GET /auth/whoami — returns user info
- POST /users — create user
- GET /users — list users
- GET /users/:id — get user

Examples and payload schemas should be generated from code or maintained here.

Signature and public_key format
--------------------------------

- signature: base64-encoded bytes of the cryptographic signature over the block `data` payload. When using Ed25519, the signature is 64 bytes and must be encoded with standard base64.
- public_key: the public key used to verify the signature. When using Ed25519, provide the raw 32-byte public key encoded as base64 (no PEM or headers).

Verification rules
------------------

- If `public_key` is provided and decodes to 32 bytes, the server will attempt Ed25519 verification of the signature against the canonical JSON serialization of the `data` field (using `serde_json::to_vec`).
- If verification fails, the server returns HTTP 400 Bad Request.
- If `public_key` is omitted or not recognized, the server will store the block but will not enforce cryptographic verification.

Examples
--------

Valid ed25519 example (pseudo):

{
	"parents": ["p1"],
	"data": {"hello":"world"},
	"signature": "<base64-encoded-64-bytes>",
	"public_key": "<base64-encoded-32-bytes>"
}

