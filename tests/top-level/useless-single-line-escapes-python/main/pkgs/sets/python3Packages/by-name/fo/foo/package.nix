# First two escapes are bad, the last is good.
{ someDrv }: builtins.seq " \. \/ \\." someDrv
