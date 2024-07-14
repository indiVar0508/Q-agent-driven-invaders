import matplotlib.pyplot as plt
import matplotlib.animation as animation
import os
import time

fig = plt.figure()
ax1 = fig.add_subplot(1, 1, 1)


def animate(i):
    if not os.path.exists("data.csv"):
        return

    pullData = open("data.csv", "r").read()
    dataArray = pullData.split("\n")[1:]
    counts = []
    best_scores = []
    xar = range(1, len(dataArray) + 1)
    for eachLine in dataArray:
        if len(eachLine) > 1:
            x, y = eachLine.split(",")
            counts.append(int(x))
            best_scores.append(int(y))
    ax1.clear()
    ax1.plot(xar, counts, label="Game Score")
    ax1.plot(xar, best_scores, label="Best Score")
    ax1.legend()


ani = animation.FuncAnimation(fig, animate, interval=500)
plt.show()
