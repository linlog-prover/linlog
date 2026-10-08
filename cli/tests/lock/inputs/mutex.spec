# Two processes and one lock: can both be in the critical section?
vars
    idle crit lock

rules
    idle >= 1, lock >= 1 ->
        idle' = idle-1,
        crit' = crit+1,
        lock' = lock-1;

    crit >= 1 ->
        crit' = crit-1,
        idle' = idle+1,
        lock' = lock+1;

init
    idle=2, crit=0, lock=1

target
    crit >= 2
