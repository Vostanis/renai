import requests

response = requests.get(
    "http://localhost:8000/stock/metrics/AAPL", 
    headers={"x-api-key": "9d207bf0-10f5-4d8f-a479-22ff5aeff8d1"}
)
print(response.text)
