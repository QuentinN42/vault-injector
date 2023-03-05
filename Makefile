.PHONY: build run
IMAGE_NAME=vault-injector

run: .env build
	@docker run -it --rm --env-file=.env $(IMAGE_NAME)

build:
	@docker build -t $(IMAGE_NAME) --progress plain .

.env: .env.example
	@cp .env.example .env
	$(info Creating .env file from .env.example)
	$(info Please edit .env file to set your environment variables)
	@exit 1
