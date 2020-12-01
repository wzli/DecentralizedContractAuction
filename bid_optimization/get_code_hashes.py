#!/usr/bin/env python3

from substrateinterface import SubstrateInterface, Keypair, SubstrateRequestException
import json

substrate = SubstrateInterface(url="http://127.0.0.1:9933",
                               ss58_format=42,
                               type_registry_preset='substrate-node-template')

contracts = substrate.iterate_map(
    module='Contracts',
    storage_function='ContractInfoOf',
)

print(json.dumps(contracts, indent=4))
