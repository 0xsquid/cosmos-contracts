# Squid Cosmos Contracts

This repo contains smart contracts which help power Squid's functionalities in the Cosmos. 

## Squid on Osmosis

The Squid contract to be deployed on Osmosis is responsible for handling two things:

1. Receive swap path and minimum output amount and execute it. 
    - Path is a sequence of pools that will be used to go from token A to token B, the same way as in the Uniswap V2 router.
2. In case of a successful swap execute specified ‘after swap action’ which can be either bank send or contract call or ibc transfer.

Since the only responsibility of this contract is to perform swaps it is stateless and does not require any ownership or pausable functions.

The contract also handles fallback scenarios for ibc-transfers, in case of packet failure or timeout contract will transfer swapped funds to the specified ‘fallback_address’.

### Testnet contract address:
```
osmo1zl9ztmwe2wcdvv9std8xn06mdaqaqm789rutmazfh3z869zcax4sv0ctqw
```
