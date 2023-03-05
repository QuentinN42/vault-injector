.PHONY: build run
IMAGE_NAME=quentinn42/vault-injector:dev
TEST_RELEASE_NAME=vault-injector
TEST_RELEASE_NAMESPACE=vault-injector

run: .env build
	@docker run -it --rm --env-file=.env $(IMAGE_NAME) bash

build:
	@docker build -t $(IMAGE_NAME) --progress plain .

deploy: build
	@docker push $(IMAGE_NAME)

.env: .env.example
	@cp .env.example .env
	$(info Creating .env file from .env.example)
	$(info Please edit .env file to set your environment variables)
	@exit 1

rollout: deploy
	@(cd k8s && helm upgrade ${TEST_RELEASE_NAME} . --install --namespace ${TEST_RELEASE_NAMESPACE} --create-namespace && kubectl rollout restart deployment ${TEST_RELEASE_NAME} -n ${TEST_RELEASE_NAMESPACE})
