{ pkgs, ... }:

{
  env.DATABASE_URL = "postgres://quasar:quasar@localhost:5432/quasar_development";

  languages.rust.enable = true;

  packages = [
    pkgs.sqlite
    pkgs.mysql80
  ];

  services.postgres = { 
    enable = true;
    listen_addresses = "127.0.0.1";
    initialScript = "CREATE USER quasar WITH SUPERUSER PASSWORD 'quasar';";
    initialDatabases = [
      { name = "quasar_development"; }
      { name = "quasar_test"; }
    ];
  };
}
