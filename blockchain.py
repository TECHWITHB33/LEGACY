
import hashlib
from hmac import digest
from numpy import block
import block as b 
import transaction
from database import Database

db = Database()

class Blockchain:

    def __init__(self) -> None:
        self.system = []
        self.temp_blocks = []
        self.temp_block : None

    
    def block_chain_len(self):
        return len(self.system)

    def add_block_to_chain(self, block):

        if (isinstance(block,b.Blocks)):
            self.system.append(block)
            
            print('block added to chain successfully....')
    
    
    def get_blocks_history(self, start_block):

       
        print('NUMBER OF BLOCK IN THE CHAIN : ', self.block_chain_len())
        for index in range(len(self.system)):
            print(f'block # {index} # HASH VALUE # {self.system[index].block_hash}')
            print('________________')
            
            # fetch block in the chain
            self.temp_block = self.system[index]
            
            # fetch valid transaction in the block
            for transaction in self.temp_block.validated_transactions:
                transaction.transaction_data()
                print("==============================================")
        
        # return self.temp_blocks

    def mine_block(self, message, dificult_level=1):
        
        if dificult_level >=1:
            prefix_string = '1' * dificult_level

            for index in range(1000):
                string_message = str(hash(message)) + str(index)
                digest = self.get_sha256(string_message)

                if digest.startswith(prefix_string):
                    print('Condition for mining meets')
                    print(f'ON  {index} iteration Nounce found # {digest}')

                    return digest
            else:
                print('VOID MINING')



    def get_sha256(self, message):
        return hashlib.sha256(message.encode('ascii')).hexdigest()



    def block_chaining(self):
       pass 



    
bc = Blockchain()
nounce = bc.mine_block('text message', 3)
# bc.
print(nounce)

#  