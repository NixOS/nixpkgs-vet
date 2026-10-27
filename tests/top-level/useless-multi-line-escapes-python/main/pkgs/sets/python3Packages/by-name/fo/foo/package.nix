{ someDrv }:
builtins.seq ''
  # Valid
  \. \/
  '''
  ''$
  ''\n
  '\n
  # Problems
  ''\\
  ''\.
'' someDrv
