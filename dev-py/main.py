from fastapi import FastAPI, Security
from fastapi.security import APIKeyHeader, APIKeyQuery
from sqlalchemy import create_engine, Column, Integer, String, Float, Date
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import json

# from security import get_api_key

DATABASE_URL = "postgresql://overseer:password@localhost/renai"

app = FastAPI()

engine = create_engine(DATABASE_URL)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
Base = declarative_base()

# async def get_db():
#     db = SessionLocal()
#     try:
#         yield db
#     finally:
#         db.close()

header_api_key = APIKeyHeader(name="x-api-key", auto_error=True)

@app.get("/stock/prices/{ticker}")
async def stock_prices(ticker: str):
    ticker = ticker.upper()
    return "boo"

@app.get("/stock/metrics/{ticker}")
async def stock_metrics(ticker: str, api_key: str = Security(header_api_key)):
    ticker = ticker.upper()
    output = engine.execute("SELECT * FROM q.stock_metrics WHERE ticker = 'NVDA'")
    return output.fetchall()
