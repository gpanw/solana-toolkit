### How to configure  
  - git clone  
  - make build-img  
  - make docker-run

### mock_onchain
  - a test on chain program written in anchor framework and deployed in DEVNET

### mock_offchain
  - a offchain program to execute the test onchain program in DEVNET to demonstrate how to c=prepare instructions to execute a smart contract

### test_validator
  - it has 3 folders
    - test_accounts --> all the account copied from mainnet required to do a swap in orca. currently it has a whirlpool id for SOL and FartCoin and all the other supporting accounts required to send a swap transaction to whirlpool smart contract
    - solana-validator.json --> configuration to run solana-test-validator locally
    - *.so --> binaries of smartcontracts dubmped from mainnet
  - to start test validator locally run `make run-solana-test-validator`

### geyser-grpc-plugin
   - the code is directly cloned from https://github.com/helius-labs/geyser-grpc-plugin
   - to comile the code run `make build-geyser-grpc-plugin`
