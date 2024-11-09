from sqlalchemy import create_engine, Column, Integer, String, Float, Date
from sqlalchemy.ext.declarative import declarative_base

Base = declarative_base()

class StockPrice(Base):
    __tablename__ = 'q.stock_prices'
    date = Column(Date)
    ticker = Column(String)
    title = Column(String)
    open = Column(Float)
    high = Column(Float)
    low = Column(Float)
    close = Column(Float)
    adj_close = Column(Float)
    volume = Column(Integer)

def serialize_stock_price(row):
    return {
        "date": row.date,
        "ticker": row.ticker,
        "title": row.title,
        "open": row.open,
        "high": row.high,
        "low": row.low,
        "close": row.close,
        "adj_close": row.adj_close,
        "volume": row.volume
    }
