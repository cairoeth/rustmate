import subprocess
import argparse

# Initialize parser
parser = argparse.ArgumentParser(
                    prog='ERC1155 Benchmark',
                    description='Run gas benchmark on ERC1155 implementations')
 
# Adding optional argument
parser.add_argument("-addr", "--address", help = "Use a specific contract address")

# Read arguments from command line
args = parser.parse_args()

def run_benchmark(address):
    functions = {
        'balanceOf(address,uint256)': ['(uint256)', address, '1'],
        'isApprovedForAll(address,address)': ['(bool)', address, address],
        'uri(uint256)': ['(string)', '1'],
        'setApprovalForAll(address,bool)': ['(bool)', address, 'true'],
        'safeTransferFrom(address,address,uint256,uint256,bytes)': ['(bool)', address, address, '1', '1', '0x'],
        'safeBatchTransferFrom(address,address,uint256[],uint256[],bytes)': ['(bool)', address, address, '[]', '[]', '0x'],
        'balanceOfBatch(address[],uint256[])': ['(uint256[])', '[]', '[]'],
        'supportsInterface(bytes4)': ['(bool)', '0x80ac58cd']
    }

    # Run estimates and write to snapshot file
    with open('.gas-snapshot', 'w') as snapshot:
        for function, args in functions.items():
            gas = subprocess.run(['cast', 'estimate', address, function + args[0]] + args[1:] + ['--rpc-url', 'http://localhost:8547'], stdout=subprocess.PIPE).stdout.decode("utf-8")[:-1]
            snapshot.write(f'ERC1155:{function} (gas: {gas})\n')

    print(subprocess.run(['cat', '.gas-snapshot'], stdout=subprocess.PIPE).stdout.decode("utf-8"))

if args.address:
    print(f"Using {args.address}...")
    address = args.address
else:
    contract = subprocess.run(['cargo', 'stylus', 'deploy', '-e', 'http://localhost:8547', '--private-key', '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'], stdout=subprocess.PIPE).stdout.decode("utf-8")
    location = contract.find('Base')
    address = contract[location-49:location-7]

run_benchmark(address)