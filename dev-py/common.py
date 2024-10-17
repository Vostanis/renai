import psycopg2
import pandas as pd

def main_price_tbl():
    conn = psycopg2.connect(
        host="localhost",
        database="renai",
        user="overseer",
        password="password",
    )

    query="""
        SELECT * FROM stock.price_view
    """

    df = pd.read_sql_query(query, conn)
    df['date'] = df['dated']

    conn.close()

    return df