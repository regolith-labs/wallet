use solana_client::nonblocking::rpc_client::RpcClient;

const RPC_URL: &str =
    "https://mainnet.helius-rpc.com/?api-key=96d65d70-28d3-449f-836f-f023cde6841e";

pub struct Gateway {
    pub rpc_client: RpcClient,
}

impl Gateway {
    pub fn new() -> Self {
        let rpc_client = RpcClient::new(RPC_URL.to_string());
        Gateway { rpc_client }
    }
}
