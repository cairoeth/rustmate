import sys
import subprocess

address = sys.argv[1]
snapshot = open('.gas-snapshot', 'w')

functions = {
    'name()': ['(string)'],
    'symbol()': ['(string)'],
    'decimals()': ['(uint8)'],
    'totalSupply()': ['(uint256)'],
    'balanceOf(address)': ['(uint256)', address],
    'allowance(address,address)': ['(uint256)', address, address],
    'nonces(address)': ['(uint256)', address],
    'approve(address,uint256)': ['(bool)', address, '100'],
    'transfer(address,uint256)': ['(bool)', address, '100'],
    'transferFrom(address,address,uint256)': ['(bool)', address, address, '100'],
    # 'permit(address,address,uint256,uint256,uint8,uint256,uint256)': ['(bool)', address, address, '100', '999', '0', '0', '0'],
    # 'domainSeparator()': ['(bytes32)'],
}

with open('.gas-snapshot', 'w') as snapshot:
    for function, args in functions.items():
        gas = subprocess.run(['cast', 'estimate', address, function + args[0]] + args[1:] + ['--rpc-url', 'http://localhost:8547'], stdout=subprocess.PIPE).stdout.decode("utf-8")[:-1]
        snapshot.write(f'ERC20:{function} (gas: {gas})\n')