#!/usr/bin/env python3

from substrateinterface import *
from substrateinterface.contracts import *

import json

substrate = SubstrateInterface(url="ws://127.0.0.1:9944",
                               ss58_format=42,
                               type_registry_preset='substrate-node-template')


keypair = Keypair.create_from_uri('//Alice')

# Upload WASM code
code = ContractCode.create_from_contract_files( metadata_file='metadata.json', wasm_file='task_auction.wasm', substrate=substrate)
receipt = code.upload_wasm(keypair)
if receipt.is_succes:
    print('* Contract WASM Uploaded')
    for event in receipt.triggered_events:
            print(f'* {event.value}')

# Deploy contract
contract = code.deploy(
        keypair=keypair, endowment=1000, gas_limit=1000000000000,
        constructor="new",
        args={'description': 1, "pay_multiplier": 1, "jury":1, "duration" : 30, "extension": 5}
        )

print(f'Deployed @ {contract.contract_address}')

contracts = substrate.iterate_map(
    module='Contracts',
    storage_function='ContractInfoOf',
)

print(json.dumps(contracts, indent=4))
