from Crypto.PublicKey import RSA 
import Crypto.Random as rand
import binascii
from Crypto.Cipher import PKCS1_OAEP
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256



class User:

	# making sure the private key is not ACCESS
	# outside the scope of the class USER...
	def __init__(self, name , email, new_user=True):
		if new_user: 
			self.generate_keys()
			self.name = name 
			self.email = email
			self.save_keys()
		else: 
			self.name = name 
			self.email = email
		
		print('super class called')


	def generate_keys(self):

		try:
			with open('public.pem') as file:
				RSA.importKey(file).read()
		except Exception:
			print('File do not exist')
			rand_val = rand.new().read
			self.private_k = RSA.generate(1024, rand_val)
			self.public_k = self.private_k.publickey()


	def identity(self):
		id = binascii.hexlify(self.public_k.export_key(format='DER')).decode('ascii')
		print('user pulich key : ' , self.public_k)
	
		return id

	def get_public_key(self):
		return self.public_k

	def set_public_key(self, public_key):
		self.public_k = public_key

	def set_private_key(self, private_key):
		self.private_k = private_key


	def __str__(self): 
		return f'Name: {self.name} Priate key (n): {self.private_k} \nPublic key: {self.public_k}'


	def encrypt(self, messages):
		# c_text is cipher text
		c_texts = []
		for message in messages:
			c_text = PKCS1_OAEP.new(self.public_k).encrypt(message.encode())
			c_texts.append(c_text)
			# print('cipher text : ' , c_text)
			
		print("Encrypted sucessfull")
		return c_texts 

	def decrypt(self , messages):
		# p-text plain text
		p_texts = [] 
		for message in messages:
			p_text =  PKCS1_OAEP.new(self.private_k).decrypt(message)
			p_texts.append(p_text.decode())
			# print('plain text : ' , p_text.decode())
		
		print("Decrypted successfull....")
		return ''.join(p_texts)


	def decrypt(self , messages, priv_key):
		# p-text plain text
		p_texts = [] 
		for message in messages:
			p_text =  PKCS1_OAEP.new(priv_key).decrypt(message)
			p_texts.append(p_text.decode())
			# print('plain text : ' , p_text.decode())
		
		print("Decrypted successfull....")
		return ''.join(p_texts)

	
	def save_keys(self):
		with open('public_key.pem', 'wb') as wr: 
			wr.write(self.public_k.export_key('PEM'))

		with open('private_key.pem', 'wb') as wr: 
			wr.write(self.private_k.export_key('PEM'))

	def load_keys(self):
		with open('public_key.pem', 'r') as rd: 
			pubkey = RSA.import_key(rd.read())

		with open('private_key.pem','r') as rd: 
			prikey = RSA.import_key(rd.read())

		return prikey, pubkey ; 

		


	# digital signing using the sender Private key
	def digital_signing(self, message):
		h = SHA256.new(message.encode())
		signed_message = pkcs1_15.new(self.private_k).sign(h)
		print('sign successfull.....', signed_message)
		return signed_message


	def authenticate_sender(self, message, sender_pub_key, signed_message):
		validation = False
		h = SHA256.new(message.encode())
		try:
			pkcs1_15.new(sender_pub_key).verify(h, signed_message)
			print('Sender Authenticated....')
			validation = True
		except (ValueError, TypeError):
			print("Invalid Sender......") 
			validation = False 

		return validation



# ===================================== EVALUATING THE CLASS ======================= 
# user = User('adam', 'binarycode1995@gmail')
# user.generate_keys()
# print(user)

# # encryptio and decryption process....
# text = 'Congratualtion this is the coolest news ever bro' # this is comming from tranc level..

# cipher = user.encrypt(text)
# print(user.decrypt(cipher))
# print('Key Type: ', type(user.private_k))

# # signing of message and verification of message.
# signed_msg = user.digital_signing(text)
# print('Signed Message : ', signed_msg)

# text = 'Congratualtion this is the coolest news ever bro' # this is comming from tranc level..
# user.authenticate_sender(text, user.get_public_key() , signed_msg)

# changing to decryptions method
# for t in text.split():
# 	c_text.append(us1.encryption(t.encode()))

# # print(c_text)

# # changing to decryptions method
# for c in c_text:
# 	p_text.append(us1.decryption(c))

# print('plain text :',   ' '.join(p_text))

# p_text = [us1.decryption() for c in c_text]

# print('PLAIN TEXT : ' , p_text)
