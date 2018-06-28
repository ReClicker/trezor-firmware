# Automatically generated by pb2py
import protobuf as p
if __debug__:
    try:
        from typing import List
    except ImportError:
        List = None
from .MultisigRedeemScriptType import MultisigRedeemScriptType


class TxOutputType(p.MessageType):
    FIELDS = {
        1: ('address', p.UnicodeType, 0),
        2: ('address_n', p.UVarintType, p.FLAG_REPEATED),
        3: ('amount', p.UVarintType, 0),  # required
        4: ('script_type', p.UVarintType, 0),  # required
        5: ('multisig', MultisigRedeemScriptType, 0),
        6: ('op_return_data', p.BytesType, 0),
        7: ('decred_script_version', p.UVarintType, 0),
        8: ('block_hash_bip115', p.BytesType, 0),
        9: ('block_height_bip115', p.BytesType, 0),
    }

    def __init__(
        self,
        address: str = None,
        address_n: List[int] = None,
        amount: int = None,
        script_type: int = None,
        multisig: MultisigRedeemScriptType = None,
        op_return_data: bytes = None,
        decred_script_version: int = None,
        block_hash_bip115: bytes = None,
        block_height_bip115: bytes = None
    ) -> None:
        self.address = address
        self.address_n = address_n if address_n is not None else []
        self.amount = amount
        self.script_type = script_type
        self.multisig = multisig
        self.op_return_data = op_return_data
        self.decred_script_version = decred_script_version
        self.block_hash_bip115 = block_hash_bip115
        self.block_height_bip115 = block_height_bip115
