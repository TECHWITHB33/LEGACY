"""
Modernized Student Identity Card Security System - Frontend
Streamlit-based client for the Rust backend API
"""

import streamlit as st
import requests
import json
from datetime import datetime
import os
from dotenv import load_dotenv

load_dotenv()

# Configuration
API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080/api/v1")
st.set_page_config(
    page_title="ID Card System",
    page_icon="🎓",
    layout="wide",
    initial_sidebar_state="expanded"
)

# Initialize session state
if "user_token" not in st.session_state:
    st.session_state.user_token = None
if "user_id" not in st.session_state:
    st.session_state.user_id = None
if "user_email" not in st.session_state:
    st.session_state.user_email = None

# ============================================================================
# API Helper Functions
# ============================================================================

def api_call(method, endpoint, data=None, authenticated=False):
    """Helper function for API calls"""
    url = f"{API_BASE_URL}{endpoint}"
    headers = {"Content-Type": "application/json"}
    
    if authenticated and st.session_state.user_token:
        headers["Authorization"] = f"Bearer {st.session_state.user_token}"
    
    try:
        if method == "GET":
            response = requests.get(url, headers=headers)
        elif method == "POST":
            response = requests.post(url, json=data, headers=headers)
        elif method == "PUT":
            response = requests.put(url, json=data, headers=headers)
        else:
            return None, "Invalid method"
        
        if response.status_code == 200 or response.status_code == 201:
            return response.json(), None
        else:
            error_data = response.json() if response.text else {"error": "Unknown error"}
            return None, error_data.get("error", "API Error")
    except Exception as e:
        return None, str(e)

# ============================================================================
# Authentication Section
# ============================================================================

def show_auth_section():
    """Display authentication UI"""
    col1, col2 = st.columns(2)
    
    with col1:
        st.subheader("📋 Register New Student")
        with st.form("registration_form"):
            email = st.text_input("Email Address", placeholder="student@university.edu")
            password = st.text_input("Password", type="password", placeholder="Min 8 characters")
            name = st.text_input("Full Name", placeholder="John Doe")
            institution = st.selectbox("Institution", [
                "Federal University of Technology Minna",
                "Obafemi Awolowo University",
                "Kwara State University",
                "University of Ilorin"
            ])
            matric_number = st.text_input("Matric Number", placeholder="MTN/SCI/001/2023")
            age = st.number_input("Age", min_value=16, max_value=100)
            gender = st.selectbox("Gender", ["Male", "Female", "Other"])
            department = st.text_input("Department", placeholder="Computer Science")
            phone = st.text_input("Phone Number", placeholder="+234-xxx-xxx-xxxx")
            
            if st.form_submit_button("Register"):
                if not all([email, password, name, institution, matric_number, department, phone]):
                    st.error("Please fill all fields")
                else:
                    payload = {
                        "email": email,
                        "name": name,
                        "institution": institution,
                        "matric_number": matric_number,
                        "age": age,
                        "gender": gender,
                        "department": department,
                        "phone_contact": phone,
                        "password": password
                    }
                    result, error = api_call("POST", "/auth/register", payload)
                    if result:
                        st.success("✅ Registration successful!")
                        st.json(result)
                    else:
                        st.error(f"Registration failed: {error}")
    
    with col2:
        st.subheader("🔐 Login")
        with st.form("login_form"):
            email = st.text_input("Email Address", placeholder="student@university.edu", key="login_email")
            password = st.text_input("Password", type="password", placeholder="Your password", key="login_password")
            
            if st.form_submit_button("Login"):
                payload = {"email": email, "password": password}
                result, error = api_call("POST", "/auth/login", payload)
                if result:
                    st.session_state.user_token = result.get("token")
                    st.session_state.user_id = result.get("user_id")
                    st.session_state.user_email = email
                    st.success("✅ Login successful!")
                    st.rerun()
                else:
                    st.error(f"Login failed: {error}")

# ============================================================================
# Main Dashboard
# ============================================================================

def show_dashboard():
    """Display authenticated user dashboard"""
    st.sidebar.title(f"👤 {st.session_state.user_email}")
    
    if st.sidebar.button("Logout"):
        st.session_state.user_token = None
        st.session_state.user_id = None
        st.session_state.user_email = None
        st.rerun()
    
    # Main navigation
    page = st.sidebar.radio(
        "Navigation",
        ["🏠 Dashboard", "🎓 Credentials", "⛓️ Blockchain", "🔐 Biometric", "💳 Payments", "📊 Audit Log"]
    )
    
    if page == "🏠 Dashboard":
        show_dashboard_page()
    elif page == "🎓 Credentials":
        show_credentials_page()
    elif page == "⛓️ Blockchain":
        show_blockchain_page()
    elif page == "🔐 Biometric":
        show_biometric_page()
    elif page == "💳 Payments":
        show_payments_page()
    elif page == "📊 Audit Log":
        show_audit_log_page()

def show_dashboard_page():
    """Dashboard overview"""
    st.header("📊 Dashboard")
    
    col1, col2, col3, col4 = st.columns(4)
    col1.metric("User ID", st.session_state.user_id[:8] + "...")
    col2.metric("Status", "Active", "✅")
    col3.metric("Credentials", "1", "+1 pending")
    col4.metric("Security", "Verified", "✅")

def show_credentials_page():
    """Credentials management"""
    st.header("🎓 Credentials")
    st.info("View and manage your identity credentials")
    
    result, error = api_call("GET", "/credentials", authenticated=True)
    if result:
        for cred in result.get("data", []):
            with st.expander(f"Credential {cred['id'][:8]}..."):
                st.json(cred)
    else:
        st.warning(f"Could not load credentials: {error}")

def show_blockchain_page():
    """Blockchain verification"""
    st.header("⛓️ Blockchain Verification")
    
    col1, col2 = st.columns(2)
    with col1:
        st.subheader("Blockchain Chain")
        if st.button("Load Blockchain"):
            result, error = api_call("GET", "/blockchain/chain")
            if result:
                st.json(result)
            else:
                st.error(f"Error: {error}")
    
    with col2:
        st.subheader("Validate Chain")
        if st.button("Validate"):
            result, error = api_call("POST", "/blockchain/validate", {})
            if result:
                st.success("✅ Blockchain is valid!")
            else:
                st.error(f"Validation failed: {error}")

def show_biometric_page():
    """Biometric authentication"""
    st.header("🔐 Biometric Authentication")
    st.info("Register or authenticate using biometric data")
    
    tab1, tab2 = st.tabs(["Register", "Authenticate"])
    
    with tab1:
        st.subheader("Register New Biometric")
        device_name = st.text_input("Device Name", placeholder="iPhone 15 Pro")
        if st.button("Start Registration"):
            result, error = api_call("POST", "/biometric/register/start", {
                "user_id": st.session_state.user_id,
                "device_name": device_name
            }, authenticated=True)
            if result:
                st.success("Registration started!")
                st.json(result)
            else:
                st.error(f"Error: {error}")
    
    with tab2:
        st.subheader("Authenticate with Biometric")
        if st.button("Start Authentication"):
            result, error = api_call("POST", "/biometric/authenticate/start", {
                "user_id": st.session_state.user_id
            }, authenticated=True)
            if result:
                st.json(result)
            else:
                st.error(f"Error: {error}")

def show_payments_page():
    """Payment system"""
    st.header("💳 Payments")
    
    col1, col2 = st.columns(2)
    
    with col1:
        st.subheader("Create Payment")
        amount = st.text_input("Amount (Stellar)")
        recipient = st.text_input("Recipient Address")
        
        if st.button("Create Payment"):
            result, error = api_call("POST", "/payments", {
                "amount": amount,
                "asset_code": "USD",
                "recipient_address": recipient
            }, authenticated=True)
            if result:
                st.success("Payment created!")
                st.json(result)
            else:
                st.error(f"Error: {error}")
    
    with col2:
        st.subheader("Payment History")
        result, error = api_call("GET", "/payments", authenticated=True)
        if result:
            for payment in result.get("data", []):
                st.write(f"💰 {payment['amount']} {payment['asset_code']} - {payment['status']}")
        else:
            st.warning(f"Could not load payments: {error}")

def show_audit_log_page():
    """Audit logging"""
    st.header("📊 Audit Log")
    
    if st.button("Refresh Audit Log"):
        result, error = api_call("GET", "/admin/audit-log", authenticated=True)
        if result:
            for log in result.get("data", []):
                st.write(f"**{log['action']}** - {log['resource_type']} - {log['timestamp']}")
        else:
            st.warning(f"Could not load audit log: {error}")

# ============================================================================
# Main Application
# ============================================================================

def main():
    st.title("🎓 Student Identity Card Security System")
    st.markdown("**Blockchain-Based Secure Identity with Biometric Authentication**")
    
    # Health check
    try:
        result, _ = api_call("GET", "../../health")
        if result and result.get("status") == "healthy":
            st.sidebar.success("✅ Backend Connected")
        else:
            st.sidebar.warning("⚠️ Backend Error")
    except:
        st.sidebar.error("❌ Backend Disconnected")
    
    if st.session_state.user_token:
        show_dashboard()
    else:
        show_auth_section()

if __name__ == "__main__":
    main()
