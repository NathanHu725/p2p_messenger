from time import time
import sys
import threading
from tqdm import tqdm
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.ticker import FormatStrFormatter
import pickle
import socket


def run_tests(host, port):
    AMOUNTS = list(range(100, 1001, 50))
    THREADS = [5, 10, 50, 100]

    search_data = pd.DataFrame(index=AMOUNTS, columns=THREADS)
    buy_data = pd.DataFrame(index=AMOUNTS, columns=THREADS)

    for threads in tqdm(THREADS):
        for amount in AMOUNTS:
            def tester(message):
                s = socket.socket()         # Create a socket object
                s.connect((host, port))
                s.sendall(message)
                s.recv(1024)
                s.close() 

            thread_list = []
            start = time()
            for _ in range(threads):
                thread_list.append(threading.Thread(target=tester, args=(b"SEND jae;joe;hahaman",)))

            for i in range(threads):
                thread_list[i].start()

            for i in range(threads):
                thread_list[i].join()
            end = time()

            search_data[threads][amount] = amount / (end - start)

            thread_list = []
            start = time()
            for _ in range(threads):
                thread_list.append(threading.Thread(target=tester, args=(b'CACHE jae;joe;hahafool',)))
            
            for i in range(threads):
                thread_list[i].start()

            for i in range(threads):
                thread_list[i].join()
            end = time()

            buy_data[threads][amount] = amount / (end - start)

    # breakpoint()
    # with open('results/concurrent/concurrent_searches.pickle', 'wb') as f:
    #     pickle.dump(search_data, f, pickle.HIGHEST_PROTOCOL)

    # with open('results/concurrent/concurrent_buys.pickle', 'wb') as f:
    #     pickle.dump(buy_data, f, pickle.HIGHEST_PROTOCOL)

    # Plot search data
    fig, ax = plt.subplots()
    for num_threads in search_data:
        ax.plot(AMOUNTS, search_data[num_threads], label=f"{num_threads}")
        ax.scatter(AMOUNTS, search_data[num_threads])

    # Set axis styles
    ax.set_xlabel("Total Send Requests", fontsize=13)
    ax.set_ylabel("Requests per Second", fontsize=13)
    ax.set_title("Send Requests", fontsize=16)
    ax.xaxis.set_major_formatter(FormatStrFormatter('%d'))
    ax.legend(title="# Concurrent Threads:")

    ax.grid(True)
    fig.tight_layout()
    plt.savefig("concurrent_send.png")

    # Clear the figure before plotting buy data
    plt.clf()
    plt.cla()
    plt.close()

    # Plot buy data
    fig, ax = plt.subplots()
    for num_threads in buy_data:
        ax.plot(AMOUNTS, buy_data[num_threads], label=f"{num_threads}")
        ax.scatter(AMOUNTS, buy_data[num_threads])

    # Set axis styles
    ax.set_xlabel("Total Cache Requests", fontsize=13)
    ax.set_ylabel("Requests per Second", fontsize=13)
    ax.set_title("Cache Requests", fontsize=16)
    ax.xaxis.set_major_formatter(FormatStrFormatter('%d'))
    ax.legend(title="# Concurrent Threads:")

    ax.grid(True)
    fig.tight_layout()
    plt.savefig("concurrent_cache.png")


if __name__ == '__main__':
    host, port = sys.argv[1], sys.argv[2]

    run_tests(host, int(port))
