pub const BOOTSTRAP_DEFAULT_PORT: u16 = 9732;
pub const BOOTSTRAP_PEERS: &[&str] = &[
    "boot.tzboot.net",
    "boot.tzbeta.net",
    "boot.mainnet.oxheadhosted.com",
];

pub const DEFAUL_IDENTITY_JSON: &str = r#"{ "peer_id": "idsfYM6UbG2nhNS1dqhsJEchaDhmd9",
  "public_key":
    "17f7d11892274a7230d969aa1335d25e637f43087b76d0e24a1a8b7d03168f5c",
  "secret_key":
    "0271fac86d020aebe6a1c9768381e7245e48e77524cca2a1652d0a621fac289f",
  "proof_of_work_stamp": "b6a4a80d765047918b037c85958c41096326a4b52ff0377e" }"#;
