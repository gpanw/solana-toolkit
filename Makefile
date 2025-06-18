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