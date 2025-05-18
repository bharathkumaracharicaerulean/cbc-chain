# CBC Chain RPC Endpoints Documentation

## System RPCs
- `system_name`: Get the node's implementation name
- `system_version`: Get the node's version
- `system_chain`: Get the chain's name
- `system_health`: Get node health info
- `system_peers`: Get connected peers
- `system_networkState`: Get network state

## Transaction Payment RPCs
- `payment_queryInfo`: Get payment info for transaction
- `payment_queryFeeDetails`: Get fee details for transaction

## CBC Custom RPCs
These endpoints are only available when `--enable-cbc-extensions` flag is enabled:

- `cbc_getProofOfInclusion`: Get proof of inclusion for data
- `cbc_getStakeInfo`: Get staking information

## Security Notes
- Some RPC methods are marked as "unsafe" and are disabled by default
- Use `--unsafe-rpc-expose` flag to enable unsafe methods
- Rate limiting is enabled by default
- Authentication is recommended for production deployments 