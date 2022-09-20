#!/usr/bin/python3
import random
import subprocess
import os

client_path = os.path.expanduser("~/bin/concordium-client")
contract_name = "mySlotMachine15"
max_rand = 9 # generate random numbers in [0, 9]

while True:
	r = random.randint(0, max_rand)
	f = open("param.json", "w")
	f.write("{\"random_value\": " + str(r) + "}")
	f.close()

	subprocess.run(["yes | concordium-client --grpc-ip service.internal.testnet.concordium.com contract update " + contract_name + " --entrypoint oracle_insert --energy 1000 --parameter-json param.json --sender Init"], shell=True)
