# This script is used to generate a large number of WSPR messages
# conforming to the WSPR spec. Callsigns are not necessarily ITU compliant
# The encoded channel symbol sequence from the reference implementation is 
# added so the wspr_rust integration test can compare its results

import random
import os
import subprocess

CHARS_ALPHA = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
CHARS_LOC = 'ABCDEFGHIJKLMNOPQR'
CHARS_NUM = '1234567890'
CHARS_ALPHANUM = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890'
CHARS_ALPHANUM_WITH_SPACE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ 1234567890'
POWERS = [0, 3, 7, 10, 13, 17, 20, 23, 27, 30, 33, 37, 40, 43, 47, 50, 53, 57, 60]

def gen_callsign():
    trailing_len = random.randint(0, 3)

    cs = []
    cs.append(random.choice(CHARS_ALPHANUM_WITH_SPACE))
    cs.append(random.choice(CHARS_ALPHA))
    cs.append(random.choice(CHARS_NUM))

    if cs[0] == ' ' and trailing_len == 0: # if first char is space, at least a third is needed
        trailing_len = 1

    for i in range(trailing_len):
        cs.append(random.choice(CHARS_ALPHA))
    return ''.join(cs)

def gen_locator():
    loc = []
    loc.append(random.choice(CHARS_LOC))
    loc.append(random.choice(CHARS_LOC))
    loc.append(random.choice(CHARS_NUM))
    loc.append(random.choice(CHARS_NUM))
    return ''.join(loc)

if __name__ == '__main__':
    try:
        os.remove("wspr.txt")
    except FileNotFoundError:
        pass

    for i in range(5000):
        callsign = gen_callsign()
        locator = gen_locator()
        power = random.choice(POWERS)
        print(callsign, locator, power)
        try:
            # somehow the wsprcode utility does not seem to encode correctly using subprocess
            # pipe everything to file
            os.system('WSPRcode "{} {} {}" >> wspr.txt'.format(callsign.strip(), locator, str(power)))
        except:
            print("Make sure WSPRCode is in this path")
            raise
    
    print('-----------------------')
    msg_chan_sym = []
    with open('wspr.txt') as f:
        for line in f:
            if 'Message:' in line:
                msg = line[9:].strip()
                while not 'Channel symbols' in line:
                    line = next(f)
                chan_sym = ''
                for i in range(6):
                    chan_sym += next(f).strip() + ' '
                while not 'Decoded message:' in line:
                    line = next(f)
                decoded = line.split(' ')[2:5]

                if not ' '.join(decoded) == msg:
                    print("!!reference encoder mismatch, skipping this msg")
                    print('  reference decoded: ' + ' '.join(decoded) + '  source: ' + msg)
                    continue
                msg_chan_sym.append(msg + ' ' + chan_sym)
    
    with open('wspr.txt', 'w') as f:
        for m in msg_chan_sym:
            f.write(m + '\n')


    
