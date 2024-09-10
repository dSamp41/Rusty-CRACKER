import pandas as pd
import platform
import subprocess
from typing import List

EXTENSION = ".exe" if platform.platform() == "Windows" else ""

def calc_times_with_nThreads(prog: str, file: str, n_threads: List[int], n_runs = 5) -> List[float]:
    avg_times = []

    for n in n_threads:
        run_times = []
        # Run the program 5 times
        for _ in range(0, n_runs):
            # Execute the command and capture the output
            result = subprocess.run([f"./release/{prog}", "--f", f"../files/{file}", "--num_thread", f"{n}"], capture_output=True, text=True)
            
            # Get the output
            time_output = result.stdout.strip()
            #print(f"Run {_}: {time_output}")
            
            # Remove the "ms" suffix and add to list
            try:
                milliseconds = int(time_output)
                run_times.append(milliseconds)
            except ValueError:
                run_times.append(0)

        # Calculate the average time in milliseconds
        total = sum(run_times)
        average = total / len(run_times)
        
        # Output the average time in milliseconds
        #print(f"{f} average time: {average:.2f} ms")
        
        avg_times.append(average)

    return avg_times




progs = ["naive", 
         "par_main_base", "par_main", "par_main_opt",
         "rayon_main_base", "rayon_main", "rayon_main_opt"
        ]


files = "syn/fixedNodes/syn_50k_2M.mtx"

N_THREADS = [0, 1, 2, 4, 8, 16]
N_RUNS = 5

info = pd.DataFrame()
info["num_threads"] = N_THREADS

info["par_base"] = calc_times_with_nThreads(progs[1], files, N_THREADS, N_RUNS)
info["par_ep"] = calc_times_with_nThreads(progs[2], files, N_THREADS, N_RUNS)
info["par_ep+os"] = calc_times_with_nThreads(progs[3], files, N_THREADS, N_RUNS)
info["rayon_base"] = calc_times_with_nThreads(progs[4], files, N_THREADS, N_RUNS)
info["rayon_ep"] = calc_times_with_nThreads(progs[5], files, N_THREADS, N_RUNS)
info["rayon_ep+os"] = calc_times_with_nThreads(progs[6], files, N_THREADS, N_RUNS)



print(info)
info.to_csv(path_or_buf="syn_50k_2M_RustvsSpark.csv", index=False)