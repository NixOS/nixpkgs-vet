{
  config,
  lib,
  pkgs,
}:

let
  cfg = config.services.bigmodule;

  configFile = lib.generators.toJSON { } cfg.settings;

  mkSystemdService = name: opts: {
    description = "${name} worker";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" ];
    serviceConfig = {
      ExecStart = "${lib.getExe cfg.package} --worker ${name} --config ${configFile}";
      User = cfg.user;
      Group = cfg.group;
      StateDirectory = "bigmodule/${name}";
      RuntimeDirectory = "bigmodule/${name}";
      Restart = "on-failure";
      RestartSec = opts.restartSec or 5;
      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      ReadWritePaths = [ "${cfg.dataDir}/${name}" ];
      LimitNOFILE = opts.maxFds or 65536;
      MemoryMax = opts.memoryMax or "2G";
      CPUQuota = opts.cpuQuota or "200%";
    };
  };
in
{
  # The `with lib;` block here has >125 non-trivia tokens, but the file is large
  # enough that the fraction stays under 25%. This tests the absolute token cap.
  options.services.bigmodule = with lib; {
    enable = mkEnableOption "bigmodule service";

    package = mkPackageOption pkgs "bigmodule" { };

    settings = mkOption {
      type = types.submodule {
        options = {
          host = mkOption {
            type = types.str;
            default = "127.0.0.1";
          };

          port = mkOption {
            type = types.port;
            default = 8080;
          };

          workers = mkOption {
            type = types.ints.positive;
            default = 4;
          };
        };
      };
      default = { };
    };

    user = mkOption {
      type = types.str;
      default = "bigmodule";
    };

    group = mkOption {
      type = types.str;
      default = "bigmodule";
    };

    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/bigmodule";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      home = cfg.dataDir;
      createHome = true;
    };

    users.groups.${cfg.group} = { };

    systemd.services.bigmodule-main = mkSystemdService "main" {
      restartSec = 5;
      maxFds = 65536;
      memoryMax = "4G";
      cpuQuota = "400%";
    };

    systemd.services.bigmodule-scheduler = mkSystemdService "scheduler" {
      restartSec = 10;
      maxFds = 1024;
      memoryMax = "512M";
      cpuQuota = "100%";
    };

    systemd.services.bigmodule-cleanup = mkSystemdService "cleanup" {
      restartSec = 60;
      maxFds = 256;
      memoryMax = "256M";
      cpuQuota = "50%";
    };

    systemd.services.bigmodule-monitor = mkSystemdService "monitor" {
      restartSec = 30;
      maxFds = 512;
      memoryMax = "128M";
      cpuQuota = "25%";
    };

    systemd.tmpfiles.rules = [
      "d ${cfg.dataDir} 0750 ${cfg.user} ${cfg.group} -"
      "d ${cfg.dataDir}/main 0750 ${cfg.user} ${cfg.group} -"
      "d ${cfg.dataDir}/scheduler 0750 ${cfg.user} ${cfg.group} -"
      "d ${cfg.dataDir}/cleanup 0750 ${cfg.user} ${cfg.group} -"
      "d ${cfg.dataDir}/monitor 0750 ${cfg.user} ${cfg.group} -"
      "d /run/bigmodule 0750 ${cfg.user} ${cfg.group} -"
    ];

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [
      cfg.settings.port
    ];

    environment.etc."bigmodule/config.json".source = configFile;

    environment.systemPackages = [ cfg.package ];

    assertions = [
      {
        assertion = cfg.settings.workers >= 1;
        message = "bigmodule requires at least 1 worker";
      }
      {
        assertion = cfg.settings.port > 0;
        message = "bigmodule port must be positive";
      }
      {
        assertion = cfg.user != "root";
        message = "bigmodule should not run as root";
      }
      {
        assertion = cfg.dataDir != "/";
        message = "bigmodule data directory must not be root";
      }
    ];
  };
}
