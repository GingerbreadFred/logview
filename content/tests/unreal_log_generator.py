#!/usr/bin/env python3
import time
import random
import sys
import os
from datetime import datetime, timezone

start_time = time.time()

def generate_log_entry():
    utc_time = datetime.now(timezone.utc).strftime('%H.%M.%S:%f')[:-3]
    
    seconds_since_boot = time.time() - start_time
    
    frame_number = random.randint(0, 998)
    
    categories = ['LogTemp', 'LogWorld', 'LogGameMode', 'LogActor', 'LogEngine', 'LogAnimation', 'LogPhysics']
    category = random.choice(categories)
    
    verbosities = ['Error', 'Warning', 'Display', 'Log', 'Verbose']
    verbosity = random.choice(verbosities)
    
    messages = [
        'Player spawned successfully',
        'Loading texture assets',
        'Physics simulation started',
        'Network connection established',
        'Memory allocation complete',
        'Rendering frame buffer',
        'Audio stream initialized',
        'Input handler registered'
    ]
    message = random.choice(messages)
    
    if verbosity in ['Error', 'Warning']:
        verbosity_part = f"{verbosity}:"
    else:
        verbosity_part = ""
    
    return f"[{utc_time}][{seconds_since_boot:.3f}][{frame_number:03d}] {category}: {verbosity_part} {message}".strip()

def main():
    try:
        default_path = os.path.join(os.path.dirname(__file__), 'generated.log')
        file_path = sys.argv[1] if len(sys.argv) > 1 else default_path
        with open(file_path, 'w') as f:
            print("Writing log file: " + file_path)
            while True:
                log_entry = generate_log_entry()
                f.write(log_entry + '\n')
                f.flush()
                
                interval = random.uniform(0.01, 0.1)
                time.sleep(interval)
    except KeyboardInterrupt:
        print("Stopping log generator...")

if __name__ == "__main__":
    main()