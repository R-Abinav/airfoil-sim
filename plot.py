import os
import glob
import pandas as pd
import matplotlib.pyplot as plt

# ── config ──────────────────────────────────────────────────────────────────
OUTPUT_DIR  = "output"
PLOTS_DIR   = "plots"
CP_ALPHA    = 5       # alpha used for the combined Cp comparison plot
MULTI_ALPHA = [0, 5, 10, 15]  # alphas shown on per-airfoil Cp overlay

os.makedirs(PLOTS_DIR, exist_ok=True)

# use ggplot; fall back to default if unavailable
try:
    plt.style.use("ggplot")
except OSError:
    pass

# ── helpers ──────────────────────────────────────────────────────────────────
def airfoil_label(stem: str) -> str:
    """convert filename stem like NACA_2414 → 'NACA 2414'."""
    return stem.replace("_", " ")

def load_cl(airfoil_stem: str) -> pd.DataFrame | None:
    path = os.path.join(OUTPUT_DIR, f"{airfoil_stem}_cl.csv")
    if not os.path.exists(path):
        return None
    return pd.read_csv(path)

def load_cp(airfoil_stem: str, alpha: int) -> pd.DataFrame | None:
    path = os.path.join(OUTPUT_DIR, f"{airfoil_stem}_alpha{alpha}_cp.csv")
    if not os.path.exists(path):
        return None
    return pd.read_csv(path)

# ── discover airfoil stems from cl CSVs ──────────────────────────────────────
cl_files = glob.glob(os.path.join(OUTPUT_DIR, "*_cl.csv"))
stems = sorted(
    os.path.basename(f).replace("_cl.csv", "") for f in cl_files
)

if not stems:
    raise FileNotFoundError(f"no *_cl.csv files found in '{OUTPUT_DIR}/'")

print(f"found airfoils: {[airfoil_label(s) for s in stems]}")

# ── plot 1: cl vs alpha — all airfoils ───────────────────────────────────────
fig, ax = plt.subplots(figsize=(8, 5))

for stem in stems:
    df = load_cl(stem)
    if df is None:
        continue
    ax.plot(df["alpha_deg"], df["cl"], marker="o", markersize=3,
            linewidth=1.5, label=airfoil_label(stem))

ax.set_xlabel("angle of attack α (deg)", fontsize=12)
ax.set_ylabel("lift coefficient $C_l$", fontsize=12)
ax.set_title("$C_l$ vs α — vortex panel method", fontsize=13)
ax.legend(fontsize=10)
ax.grid(True)
fig.tight_layout()
fig.savefig(os.path.join(PLOTS_DIR, "cl_vs_alpha.png"), dpi=150)
plt.close(fig)
print("saved: plots/cl_vs_alpha.png")

# ── plot 2: cp vs x/c at alpha = CP_ALPHA — all airfoils ────────────────────
fig, ax = plt.subplots(figsize=(8, 5))

for stem in stems:
    df = load_cp(stem, CP_ALPHA)
    if df is None:
        print(f"  warning: no cp file for {airfoil_label(stem)} at α={CP_ALPHA}°")
        continue
    ax.plot(df["x_over_c"], df["cp"], linewidth=1.5, label=airfoil_label(stem))

ax.invert_yaxis()  # aerodynamic convention — suction peak points up
ax.set_xlabel("x/c", fontsize=12)
ax.set_ylabel("pressure coefficient $C_p$", fontsize=12)
ax.set_title(f"$C_p$ vs x/c at α = {CP_ALPHA}°", fontsize=13)
ax.legend(fontsize=10)
ax.grid(True)
fig.tight_layout()
fig.savefig(os.path.join(PLOTS_DIR, f"cp_vs_xc_alpha{CP_ALPHA}.png"), dpi=150)
plt.close(fig)
print(f"saved: plots/cp_vs_xc_alpha{CP_ALPHA}.png")

# ── plot 3: per-airfoil cp overlay at multiple alphas ────────────────────────
for stem in stems:
    fig, ax = plt.subplots(figsize=(8, 5))

    for alpha in MULTI_ALPHA:
        df = load_cp(stem, alpha)
        if df is None:
            print(f"  warning: no cp file for {airfoil_label(stem)} at α={alpha}°")
            continue
        ax.plot(df["x_over_c"], df["cp"], linewidth=1.5, label=f"α = {alpha:+d}°")

    ax.invert_yaxis()
    ax.set_xlabel("x/c", fontsize=12)
    ax.set_ylabel("pressure coefficient $C_p$", fontsize=12)
    ax.set_title(f"{airfoil_label(stem)} — $C_p$ at multiple α", fontsize=13)
    ax.legend(fontsize=10)
    ax.grid(True)
    fig.tight_layout()

    fname = f"{stem}_cp_multi_alpha.png"
    fig.savefig(os.path.join(PLOTS_DIR, fname), dpi=150)
    plt.close(fig)
    print(f"saved: plots/{fname}")

print("\ndone. all plots in 'plots/'")
