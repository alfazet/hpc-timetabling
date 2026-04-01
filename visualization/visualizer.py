import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import matplotlib
matplotlib.use("TkAgg")

fig, ax = plt.subplots(2)

line_best_h, = ax[0].plot([], [], label="best_h")
line_avg, = ax[0].plot([], [], label="mean_h")
line_best_s, = ax[1].plot([], [], label="best_s")
# line_avg, = ax[1].plot([], [], label="mean_s")

ax[0].set_xlabel("generation")
ax[1].set_xlabel("generation")
ax[0].set_ylabel("min. penalty")
ax[1].set_ylabel("min. penalty")
ax[0].legend()
ax[1].legend()

def update(frame):
    try:
        df = pd.read_csv("metrics.csv", names=["gen","best_h","best_s","mean"])

        x = df["gen"]

        line_best_h.set_data(x, df["best_h"])
        line_best_s.set_data(x, df["best_s"])
        line_avg.set_data(x, df["mean"])
        # line_mut.set_data(x, df["mut"])

        # rescale axes dynamically
        for a in ax:
            a.relim()
            a.autoscale_view()

    except Exception:
        pass

    return line_best_h, line_best_s, line_avg#, line_mut

ani = FuncAnimation(fig, update, interval=100)
plt.show()
