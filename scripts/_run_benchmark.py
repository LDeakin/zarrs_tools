
import platform
import subprocess

def clear_cache():
    if platform.system() == "Darwin":
        subprocess.call(['sync', '&&', 'sudo', 'purge'])
    elif platform.system() == "Linux":
        subprocess.call(['sudo', 'sh', '-c', "sync; echo 3 > /proc/sys/vm/drop_caches"])
    else:
        raise Exception("Unsupported platform")

def time_args():
    if platform.system() == "Darwin":
        return ["gtime", "-v"]
    elif platform.system() == "Linux":
        return ["time", "-v"]
    else:
        raise Exception("Unsupported platform")
