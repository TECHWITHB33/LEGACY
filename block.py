
class Blocks:
    pass


class Block(Blocks):

    last_block_hash : None

    def __init__(self, block_capacity=3) -> None:

        # main variable
        self.previous_block_hash = ''
        self.block_hash = ''
        self.nounce = ''
        self.validated_messages = []
        

        # other intance variable
        self.block_capacity = block_capacity
    

    # checking if validated transaction is full
    def is_ready_for_mining(self):
        
        if len(self.validated_messages) <= self.block_capacity:
            return False 
        else: 
            return True


class GenesisBlock(Blocks):
    # last_block_hash : None
    def __init__(self) -> None:
        self.previous_block_hash = None
        self.block_hash = ''
        self.nounce = None
        self.validated_messages = []



