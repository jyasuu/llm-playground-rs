# 🚀 Real API Integration - Setup Guide

## ✅ **Real API Integration Implemented!**

The LLM Playground now supports **real API calls** to both Gemini and OpenAI-compatible services!

---

## 🔧 **How to Set Up APIs**

### **1. Gemini API Setup**

#### Get Your API Key:
1. Go to [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Click "Create API Key"
3. Copy the generated API key

#### Configure in App:
1. Open the LLM Playground at http://127.0.0.1:8080/
2. Click "Settings" button in sidebar
3. Ensure "Gemini API" is selected
4. Paste your API key in the "API Key" field
5. Select your preferred model (gemini-1.5-pro, gemini-1.5-flash, etc.)
6. Click "Save Configuration"

### **2. OpenAI API Setup**

#### Get Your API Key:
1. Go to [OpenAI Platform](https://platform.openai.com/api-keys)
2. Click "Create new secret key"
3. Copy the generated API key

#### Configure in App:
1. Open Settings panel
2. Select "OpenAI-compatible API"
3. Enter API URL: `https://api.openai.com/v1`
4. Paste your API key
5. Select model (gpt-4o, gpt-4-turbo, gpt-3.5-turbo)
6. Click "Save Configuration"

---

## 🎯 **Testing the Integration**

### **Quick Test Steps:**
1. **Configure API**: Add your API key in Settings
2. **Create Session**: Click "+" to create a new chat session
3. **Send Message**: Type "Hello! Can you help me?" and press Enter
4. **Check Console**: Open browser console (F12) to see detailed logs
5. **Get Response**: You should receive a real response from the LLM!

### **Expected Behavior:**
- ✅ **With Valid API Key**: Real responses from Gemini/OpenAI
- ❌ **Without API Key**: Friendly error message asking to configure
- ❌ **Invalid API Key**: Clear error message with troubleshooting info
- ⚠️ **Network Issues**: Helpful error messages with suggestions

---

## 🔍 **Debugging Features**

### **Console Logging:**
The app now provides detailed console logs:
```
Send button clicked!
Gemini API call started
API key present, processing messages...
Making request to: https://generativelanguage.googleapis.com/v1beta/models/...
Request body: {"contents":[...],"generationConfig":{...}}
Response status: 200
```

### **Error Handling:**
- **Invalid API Keys**: Clear messages with links to get new keys
- **Rate Limits**: Helpful advice about waiting and usage limits
- **Network Errors**: Guidance on connectivity and configuration
- **Quota Issues**: Information about API usage limits

---

## ⚙️ **Advanced Configuration**

### **System Prompts:**
- Configure custom system prompts in Settings
- Applied to all conversations automatically
- Useful for setting LLM behavior and context

### **Parameters:**
- **Temperature**: Controls response creativity (0.0-1.0)
- **Max Tokens**: Limits response length
- **Retry Delay**: Time between retry attempts

### **Model Selection:**
- **Gemini**: gemini-1.5-pro, gemini-1.5-flash, gemini-1.0-pro
- **OpenAI**: gpt-4o, gpt-4-turbo, gpt-3.5-turbo
- **Custom Endpoints**: Support for OpenAI-compatible APIs

---

## 🚨 **Common Issues & Solutions**

### **"Please configure your API key"**
- **Solution**: Add your API key in Settings panel
- **Check**: Make sure the key is for the correct provider (Gemini vs OpenAI)

### **"Invalid API key"**
- **Solution**: Verify the API key is correct and active
- **Check**: Copy-paste the key carefully, no extra spaces

### **"Network error"**
- **Solution**: Check internet connection
- **Check**: Some networks block API requests (try different network)

### **"Rate limit exceeded"**
- **Solution**: Wait a few seconds and try again
- **Check**: Consider upgrading API plan for higher limits

### **No response/Loading forever**
- **Solution**: Check browser console for error details
- **Check**: Verify API endpoint URLs are correct

---

## 🎉 **What's Working Now**

✅ **Real Gemini API Integration**
- Full conversation context
- System prompt support
- Proper error handling
- Usage logging

✅ **Real OpenAI API Integration**
- Compatible with OpenAI and clones
- Custom endpoint support
- Standard chat completions format

✅ **Smart Error Handling**
- User-friendly error messages
- Detailed logging for debugging
- Graceful degradation

✅ **Configuration Persistence**
- API keys saved locally
- Settings persist across sessions
- Easy switching between providers

---

## 🔄 **Next Steps Available**

Now that real API integration is working, you can:

1. **Test with Real APIs**: Try both Gemini and OpenAI
2. **Function Calling**: Add tool/function execution
3. **Streaming Responses**: Real-time response streaming
4. **Enhanced Features**: Message editing, retry, export
5. **Multiple Models**: Side-by-side model comparison

**The playground is now fully functional with real LLM APIs!** 🎊