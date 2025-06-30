IMG_NAME ?= solana_toolkit
APP_NAME ?= solana_arbitrage_bot

build-img:
	docker build -t $(IMG_NAME) .

docker-run:
	docker run -d -v ./src:/usr/src/myapp --env-file .env --name $(APP_NAME) $(IMG_NAME)

docker-exec:
	docker exec -it $(APP_NAME)  /bin/bash

docker-stop:
	docker stop $(APP_NAME) 

docker-rm:
	docker rm $(APP_NAME) 

docker-rmi:
	docker rmi $(IMG_NAME)

build-mock-onchain:
	docker exec $(APP_NAME) sh -c "cd mock_onchain && anchor build"

deploy-mock-onchain:
	docker exec $(APP_NAME) sh -c "cd mock_onchain && anchor deploy"

run-mock-offchain:
	docker exec $(APP_NAME) sh -c "cd mock_offchain && cargo run"

build-geyser-grpc-plugin:
	docker exec $(APP_NAME) sh -c "cd geyser-grpc-plugin && cargo build --release"

run-solana-test-validator:
	docker exec $(APP_NAME) sh -c "cd test_validator && \
	solana-test-validator --account-dir test_accounts/ --bpf-program whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc whirlpool.so --reset --geyser-plugin-config solana-validator.json"