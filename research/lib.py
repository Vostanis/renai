# Out-of-the-box functions for ease of use

import requests
import pandas as pd
# import seaborn as sns
# import matplotlib.pyplot as plt
import numpy as np
import os # env vars
from dotenv import load_dotenv

dotenv_path = os.path.join(os.path.dirname(__file__), '..', '.env')
load_dotenv(dotenv_path)

BASE = os.getenv("DATABASE_URL")

class Index:
    def stock(self):
        url = BASE + "/stock/us_index"
        df = requests.get(url).json()
        df = pd.json_normalize(df["data"])
        return df

class Stock:
    #   <todo!>
    #   
    #   1. a. `.plot()` - generic X over time graphs
    #   2. `.ml()` - price & core combined into 1 dataset
    def __init__(self, ticker):
        self.ticker = ticker

    # def __str__(self):
    #     return f\"{self}\"

    # fetch the data from the CouchDB database, using the ticker intialised.
    # ```python
    # stock = Stock(ticker)
    # price = stock.price()
    # price['Volume']
    # ```
    def fetch(self):
        url = BASE + "/stock/" + self.ticker
        df = requests.get(url).json()
        return df["data"]

    def core(self, ml=False):
        df = self.fetch()
        df = pd.DataFrame(df["core"])

        df['dated'] = pd.to_datetime(df['dated'])
        df = df.set_index(df["dated"])
        df = df.drop(columns=["dated"])
        # <--- forward fill empty values based on number of occurrences

        # COLUMNS
        # ====================================================
        # US interest rate
        # US unemployment
        # sentiment analysis
        # ffill based on differences

        # output y: max, absolute, pct change
        
        return df
    
    def price(self):
        df = self.fetch()
        df = pd.DataFrame(df["price"])

        df['dated'] = pd.to_datetime(df['dated'])
        df = df.set_index(df["dated"])
        df = df.drop(columns=["dated"])

        # COLUMNS
        # ====================================================
        # percentage change
        df['pct'] = df['adj_close'].pct_change().fillna(0)

        # earnings date


        # std deviation of volume
    

        # fetch news sentiment for volume spikes


        return df

    # def time_decay()
        # <--- measure decrease of volatility after huge volume spikes

    # bug: not every core metric matches on a price date (might not be an issue)
    # add: volatility
    # add: us interest rate & unemployment
    def ml(self):
        price = self.price()
        core = self.core()
        ml = price.join(core, on='dated')
        ml = ml.drop(columns=['close', 'high', 'low', 'open'])
        ml = ml.ffill().fillna(0).diff().fillna(0)
        ml['adj_close'] = price['adj_close']
        ml['volume'] = price['volume']
        ml['pct'] = price['pct']

        # COLUMNS
        # ====================================================
        # earnings date
        dates = core.drop(columns=['EntityCommonStockSharesOutstanding', 'EntityPublicFloat']).dropna(how='all').index
        ml['is_earnings_date'] = ml.index.isin(dates)
        ml.index = pd.to_datetime(ml.index)

        # relative earnings date
        ml['earnings_date'] = pd.to_datetime(np.where(ml['is_earnings_date'], ml.index, None))
        ml['earnings_date'] = ml['earnings_date'].ffill()

        # days since the latest earnings were released
        ml['days_since_earnings'] = (ml.index - ml['earnings_date']).dt.days
        ml = ml.dropna(subset='earnings_date')

        # quarter: which quarter of the year does this belong to?
        ml['quarter'] = ml['earnings_date'].dt.month.apply(
            lambda x:   4 if x in [1, 2]
                else    1 if x in [4, 5]
                else    2 if x in [7, 8]
                else    3 if x in [10, 11]
                else    None
        )

        # next earnings date
        dates = pd.DataFrame(dates) # <--- re-using the `core` dates from `earnings date boolean`
        dates['next_earnings_date'] = dates['dated'].shift(-1)
        ml = pd.merge(ml, dates, on='dated', how='left')
        ml['dated'] = pd.to_datetime(ml['dated'])
        ml = ml.set_index('dated')
        ml['next_earnings_date'] = pd.to_datetime(ml['next_earnings_date'].ffill())

        # days until next earnings
        max_days_since_earnings = max(ml['days_since_earnings'])
        ml['days_until_earnings'] = ml.apply(
            lambda row:
                        # if `earnings_date` == `next_earnings_date`, use the max(`days_since_earnings`) to estimate the actual `next_earnings_date`
                        max(0, ((row['earnings_date'] + pd.Timedelta(days=max_days_since_earnings)) - pd.to_datetime(row.name)).days)
                if      row['earnings_date'] == row['next_earnings_date']

                        # otherwise, calculate the difference between the `next_earnings_date` and the row's `date`
                else    (row['next_earnings_date'] - pd.to_datetime(row.name)).days 
                if      row['earnings_date'] != row['next_earnings_date']

                        # capture errors
                else    None,
            axis=1
        )

        return ml