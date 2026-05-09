import os
import pandas as pd
import matplotlib.pyplot as plt

def main():
    # Ensure plots directory exists
    os.makedirs("plots", exist_ok=True)

    # Use seaborn-v0_8 style if available
    try:
        plt.style.use("seaborn-v0_8")
    except OSError:
        pass

    mapping = {
        "CLARK Y AIRFOIL": ("output/CLARK_Y_AIRFOIL_cl.csv", "xflr5_data/clark_y.txt"),
        "E387": ("output/E387_cl.csv", "xflr5_data/e387.txt"),
        "NACA 2414": ("output/NACA_2414_cl.csv", "xflr5_data/NACA_2414.txt"),
        "NACA 4412": ("output/NACA_4412_cl.csv", "xflr5_data/NACA_4412.txt")
    }

    fig, axes = plt.subplots(2, 2, figsize=(10, 8), sharex=True, sharey=True)
    axes = axes.flatten()

    for idx, (name, (rust_path, xflr5_path)) in enumerate(mapping.items()):
        ax = axes[idx]
        
        # Rust Output
        if os.path.exists(rust_path):
            df_rust = pd.read_csv(rust_path)
            ax.plot(df_rust["alpha_deg"], df_rust["cl"], color="blue", linestyle="-", label="Panel Method (Rust)")
        else:
            print(f"Warning: Missing {rust_path}")
            
        # XFLR5 Output
        if os.path.exists(xflr5_path):
            # Skip 11 header lines, separator is whitespace
            df_xflr5 = pd.read_csv(xflr5_path, sep=r'\s+', skiprows=11, header=None)
            ax.plot(df_xflr5[0], df_xflr5[1], color="red", linestyle="--", label="XFLR5 (XFoil)")
        else:
            print(f"Warning: Missing {xflr5_path}")
            
        ax.set_title(name)
        ax.grid(True)
        ax.set_xlim(-5, 15)
        ax.set_xlabel("alpha (degrees)")
        ax.set_ylabel("Cl")

    # Shared legend
    handles, labels = axes[0].get_legend_handles_labels()
    if handles:
        fig.legend(handles, labels, loc='lower center', ncol=2, bbox_to_anchor=(0.5, -0.05))

    plt.tight_layout()
    # Save the figure
    out_path = os.path.join("xflr5_sim_plot", "comparison_cl.png")
    fig.savefig(out_path, bbox_inches='tight')
    plt.close(fig)
    print(f"Plot saved to {out_path}")

if __name__ == "__main__":
    main()
