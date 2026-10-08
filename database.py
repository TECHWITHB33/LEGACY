import sqlite3 as sq
from Crypto.PublicKey import RSA 
import streamlit as st 

class  Database: 


    def __init__(self) -> None:
        
        with sq.connect('Qrblockchian.db')  as db :
            Database.database = db
            self.data = db
            print('Database connected successfully')

# =============================================================== USER BLOCKCHAIN SECTION =========================================================
  # name varhcar not null, 
                    # institute varchar not null, 
                    # matric varchar not null, 
                    # age int not null, 
                    # department varchar not null, 
                    # contact varchar not null
    def create_student_table(self):
        query = '''
                CREATE TABLE IF NOT EXISTS student_id (
                    public_address BLOB  NOT NULL,
                    sign_data BLOB NOT NULL, 
                    qr_image BLOB NOT NULL, 
                    passport BLOB NOT NULL, 
                    user_data text not null
              
                )
                '''
        self.data.execute(query)

    def insert_to_user_table(self, pubkey, sign_data , qr_image, passport, user_data): 
        query = '''
                INSERT INTO student_id VALUES (?, ?, ?, ?, ?)
                '''

        self.data.execute(query , (sq.Binary(pubkey), sq.Binary(sign_data),sq.Binary(qr_image), sq.Binary(passport), user_data))
        self.data.commit()
        print('student ID added successfully')

    def get_student_id_record(self): 
        query = ''' SELECT sign_data , passport , qr_image, user_data FROM student_id ;'''

        data  = self.data.execute(query)
        return data.fetchall()



# =============================================================== USER BLOCKCHAIN SECTION =========================================================
  # name varhcar not null, 
                    # institute varchar not null, 
                    # matric varchar not null, 
                    # age int not null, 
                    # department varchar not null, 
                    # contact varchar not null
    def create_blockchain_table(self):
        query = '''
                CREATE TABLE IF NOT EXISTS blockchain (
                    block_num int primary key NOT NULL,
                    pre_hash varchar NOT NULL, 
                    hash varchar NOT NULL, 
                    nounce varchar NOT NULL
              
                )
                '''
        self.data.execute(query)


    def insert_to_blockchain_table(self, id, pre_hash, hash, nounce): 
            query = '''
                    INSERT INTO blockchain VALUES (?, ?, ?, ?)
                    '''

            self.data.execute(query , (id, pre_hash, hash, nounce))
            self.data.commit()
            print('student ID added successfully')

    def get_blockchain_id_record(self): 
        query = ''' SELECT * FROM blockchain ;'''

        data  = self.data.execute(query)
        return data.fetchall()