run:
	cargo run -- --start-date "2023-01-01" --end-date "2023-12-31" --loan-amount 1000.00 --loan-currency GBP --base-interest-rate 5.0 --margin 1.5

docker_build:
	docker build -t oneiro:latest .

docker_test: docker_build
	docker run oneiro:latest --start-date "2023-01-01" --end-date "2023-12-31" --loan-amount 1000.00 --loan-currency GBP --base-interest-rate 5.0 --margin 1.5

docker_run: docker_build
	docker run oneiro:latest

