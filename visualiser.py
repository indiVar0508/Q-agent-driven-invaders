import matplotlib.pyplot as plt
import matplotlib.animation as animation
import os
import seaborn as sns

fig = plt.figure()
ax1 = fig.add_subplot(1, 2, 1)
ax2 = fig.add_subplot(1, 2, 2)


def animate(i):
    if not os.path.exists("data.csv"):
        return
    try:
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
        ax1.bar(xar, best_scores, label="Best Score", color="orange")
        ax1.set_title("Game vs Score")
        ax1.legend()
    except Exception:
        pass
    try:
        pullData = open("q_table.csv", "r").read()
        dataArray = pullData.split("\n")[1:]
        matrix = []
        for eachLine in dataArray:
            if len(eachLine) > 1:
                matrix.append(
                    list(map(lambda x: float(x), eachLine.strip(",").split(",")))
                )
        ax2.set_title("Q Agent Brain matrix")
        sns.heatmap(
            matrix,
            ax=ax2,
            cbar=False,
            xticklabels=["Left", "Right", "Shoot"],
            linewidths=1,
        )
        # ax2.imshow(matrix, cmap="hot", interpolation="nearest", aspect="auto")
        # ax2.set_xticks([0, 1, 2], labels=["Left", "Right", "shoot"])

    except Exception:
        pass


ani = animation.FuncAnimation(fig, animate, interval=3000)
plt.show()
