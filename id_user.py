# -*- coding: utf-8 -*-
"""
Created on Fri Jan  6 09:58:31 2023

@author: HP
"""

from user import User
import json
from Crypto.PublicKey import RSA 
import Crypto.Random as rand
import binascii
from Crypto.Cipher import PKCS1_OAEP
from Crypto.Signature import pkcs1_15
from Crypto.Hash import SHA256


class IdUser(User):

    def __init__(self, email, institute, matric, name, age, sex , dept, contact): 
        self.institute = institute
        self.matric = matric
        self.age = age
        self.sex = sex 
        self.dept = dept 
        self.contact = contact
        super().__init__(self, name, email)
        self.name = name 
        self.email = email
        print('sub class called')
        


    def __str__(self):
        user_data = {
            'email': self.email,
            'name': self.name, 
            'institute' : self.institute,
            'matric': self.matric, 
            'age': self.age, 
            'sex': self.sex, 
            'departement': self.dept, 
            'contact':self.contact
        }

        str_json = json.dumps(user_data)
        return str_json


    # def encrypt(self, messages):
    #     c_text = PKCS1_OAEP.new(self.public_k).encrypt(messages.encode())
    #     return c_text

        
# id_us = IdUser('adam', 'adam@gmail.com')
