"""Created on Thu Jan  5 22:12:13 2023

@author: binary..
"""
import streamlit as st 
from id_user import IdUser
import qrcode
import json
import cv2
import numpy as np
from io import BytesIO
from cv2 import VideoCapture, waitKey, destroyWindow
from pyzbar.pyzbar import decode
from Crypto.PublicKey import RSA 
from blockchain import Blockchain
from database import Database
from block import Block, GenesisBlock
st.set_page_config(page_title=None, page_icon=None, layout="centered", initial_sidebar_state="auto", menu_items=None)


 
# with open('style.css') as css: 
#     st.markdown(f'<style>{css.read()}</style>', unsafe_allow_html=True)

def update_block_chain(isGenesis = False):
    block = None 
    students_ids = db.get_student_id_record()
    chain_record = db.get_blockchain_id_record()
    last_id = students_ids[-1]
    chain = Blockchain()
    chain.system = []
    # st.write(len(chain_record))
    # get last hash from the block table

    if len(chain_record) == 0: 
        st.write('Genesis block')
        block = GenesisBlock()
        block.validated_messages.append(last_id)
        block.block_hash = hash(last_id)
        # creaing the block chain
        nounce = chain.mine_block(block)

        block.nounce = nounce 
        # don't forget to call the last has ..  for non genesis block.. 
        
        chain.add_block_to_chain(block)
        chain_db_index = len(chain_record)
        db.insert_to_blockchain_table(chain_db_index, block.block_hash , block.block_hash, block.nounce)

    else: 
        last_chain_record = chain_record[-1]
        
        block = Block(2)
        block.validated_messages.append(last_id)
        block.block_hash = hash(last_id)
        block.previous_block_hash = block.block_hash

    
        # creaing the block chain
        nounce = chain.mine_block(block)
        block.nounce = nounce 

        # don't forget to call the last has ..  for non genesis block.. 
        
        chain.add_block_to_chain(block)
        chain_db_index = len(chain_record)
        db.insert_to_blockchain_table(chain_db_index, last_chain_record[1] , block.block_hash, block.nounce)
    # st.write(block.block_hash)
    # st.write(block.previous_block_hash)
    # st.write(block.nounce)

    # st.write(type(block.block_hash))
    # st.write(type(block.previous_block_hash))
    # st.write(type(block.nounce))




db = Database()
db.create_student_table()
db.create_blockchain_table()

if len(db.get_blockchain_id_record()) == 0:
    update_block_chain()



def decoder(image):
    gray_img = cv2.cvtColor(image,0)
    barcode = decode(gray_img)

    for obj in barcode:
        points = obj.polygon
        (x,y,w,h) = obj.rect
        pts = np.array(points, np.int32)
        pts = pts.reshape((-1, 1, 2))
        cv2.polylines(image, [pts], True, (0, 255, 0), 3)

        barcodeData = obj.data.decode("utf-8")
        barcodeType = obj.type
        string = "Data " + str(barcodeData) + " | Type " + str(barcodeType)
        
        cv2.putText(frame, string, (x,y), cv2.FONT_HERSHEY_SIMPLEX,0.8,(255,0,0), 2)
        print("Barcode: "+barcodeData +" | Type: "+barcodeType)


# def create_qr_code(image_name , data, fcolor='white', bcolor='black'):
#     qr = qrcode.QRCode(
#     version=20,
#     error_correction=qrcode.constants.ERROR_CORRECT_L,
#     box_size=10,
#     border=4,
#     )
#     qr.add_data(data)
#     qr.make(fit=True)
#     img = qr.make_image(fill_color=fcolor, back_color=bcolor)
#     qr_image = img.save(image_name)
#     im.image(image_name)
#     return qr_image

# def download_qr_code():
#     pass

# def read_qr_code(img_path):
#     img=cv2.imread("medium.png")
#     det=cv2.QRCodeDetector()
#     val, pts, st_code=det.detectAndDecode(img)
#     st.write(val)
#     return val

st.header('QrBlockChain ID\'s Generator')


reg_id_tab , auth_id_tab = st.tabs(['Register ID' , 'Authentication'])


with reg_id_tab: 
    st.subheader('Register ID Blockchain')
    
    stud_info_col , img_col = st.columns([2,3])
    email_text = stud_info_col.text_input('Email', placeholder='Sampe@gmail.com')
    intitute_opt = stud_info_col.selectbox('Select intitution', ['fut minna', 'OAU', 'KWASU', 'Unilorin'], )
    matric_txt = stud_info_col.text_input('Enter Matric')
    fullname_txt = stud_info_col.text_input('Full Name', placeholder='firstname    othername   middlename')
    age_txt = stud_info_col.text_input('Enter Age')
    sex_text = stud_info_col.selectbox('Sex', ['male', 'female'])
    dept_txt = stud_info_col.text_input('Department')
    contact_txt = stud_info_col.text_input('Phone No.', placeholder='+234-xxx-xxx-xxx')
    
    # row two
    im_pane= img_col.image('usericon.jpg')
    passport = img_col.file_uploader("Upload Passport", type=["png", "jpg", 'jpeg'], key="passport")

    


    if passport:
        with open('passport.png', 'wb') as im_file : 
            im_file.write(passport.getbuffer())


        passport_loader = None 
        with open('passport.png', 'rb') as im_file : 
            passport_loader = im_file.read()

        im_pane.image(passport, width=150)

    if stud_info_col.button('Register Blockchain ID'): 
        data = [email_text, intitute_opt, matric_txt, fullname_txt, age_txt, sex_text, dept_txt, contact_txt]
        data = [str(d).strip().lower() for d in data]

        if '' not in data: 
         

            # creating ID user option 
            id_holder_obj = IdUser(data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7])
            label_holder = ['email', 'institute', 'matric', 'fullname', 'age', 'gender', 'department', 'contact']
            
            json_data = {}
            for index in range(len(data)):
                json_data[label_holder[index]] = data[index]  

            str_json_data = json.dumps(json_data)
            qr = qrcode.QRCode(
            version=10,
            error_correction=qrcode.constants.ERROR_CORRECT_L,
            box_size=10,
            border=4,
            )
            qr.add_data(id_holder_obj.get_public_key().export_key())
            qr.make(fit=True)
            img = qr.make_image(fill_color="black", back_color="aqua")
            img.save("qrcode.png")
            img_col.image('qrcode.png')


            sign_data = id_holder_obj.digital_signing(id_holder_obj.__str__())
          
            buf = BytesIO()
            img.save(buf, format="JPEG")
            byte_im = buf.getvalue()
            btn = st.download_button(
            label="Download QR",
            data=byte_im,
            file_name="qrcode.png",
            mime="image/jpeg",
            )

            with open('id_qr.png', 'rb') as pngfile: 
                img_file = pngfile.read()
             
            # inserting record into database
            db.insert_to_user_table(id_holder_obj.get_public_key().export_key(), sign_data, img_file , passport_loader , id_holder_obj.__str__())
            st.success('Registration Complete')
            update_block_chain()

        else: 
            img_col.error('Provide All details')


    
with auth_id_tab: 
    st.subheader('Authentication ID\'s Holder' )
    private_key_text = st.text_area('Enter public key', placeholder='Enter Public Key', height=300)
    
    
    if st.button('Validate'):
        if private_key_text != '':
            rsa_key = RSA.import_key(private_key_text.strip())
            # rsa_key = private_key_text.strip()
            # st.write(rsa_key)
            pull_record = db.get_student_id_record()
            confired_record = []
            for record in pull_record: 
                signer_record = record[0]
                user_data = record[-1]
                import user
                us = user.User('' ,'' , new_user=False)
                valid = us.authenticate_sender(user_data, rsa_key , signer_record)
                if valid: 
                    # st.write(f'record find {user_data}')
                    st.success('Digital Signature Confirm')
                    confired_record.append(record)
            # key can be use for decryption .. 
            if confired_record != []:
                record = confired_record[-1]
                col1 , col2 = st.columns([3,3])
                # pub_address = record[0]
                pssport = record[1]
                qr_im = record[2]
                us_data = record[3]
                json_dt = json.loads(user_data)
                # st.write(json_dt)
                col1.image(pssport, width=200)
                for key, val in json_dt.items(): 
                    col1.markdown(f'<li> <b>{key}</b> : {val} </li>' , unsafe_allow_html=True)



                qr = qrcode.QRCode(
                version=10,
                error_correction=qrcode.constants.ERROR_CORRECT_L,
                box_size=10,
                border=4,
                )
                qr.add_data(private_key_text.strip())
                qr.make(fit=True)
                img = qr.make_image(fill_color="black", back_color="aqua")
                img.save("dbqrcode.png")
                col2.image('dbqrcode.png', width=400)

                buf = BytesIO()
                img.save(buf, format="JPEG")
                byte_im = buf.getvalue()
                btn = st.download_button(
                label="Download QR",
                data=byte_im,
                file_name="dbqrcode.png",
                mime="image/jpeg",
                )
        else: 
            st.error('Suply Public Key')
        
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    