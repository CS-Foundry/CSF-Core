# Changelog

All notable changes to CSF-Core will be documented in this file.

## [0.0.3](https://github.com/CS-Foundry/CSF-Core/compare/v0.0.2...v0.0.3) (2026-01-02)


### Bug Fixes

* pipeline build error ([8007bc4](https://github.com/CS-Foundry/CSF-Core/commit/8007bc47a90f049421f4d0a7d420424bab969e03))

## [0.0.2](https://github.com/CS-Foundry/CSF-Core/compare/v0.0.1...v0.0.2) (2026-01-02)


### Bug Fixes

* pipeline ([7a0154d](https://github.com/CS-Foundry/CSF-Core/commit/7a0154d9b71931db881783b179f599316d44ce9e))

# [1.1.0](https://github.com/CS-Foundry/CSF-Core/compare/v1.0.0...v1.1.0) (2026-01-02)


### Features

* lint pipeline and fix builds ([78f9dea](https://github.com/CS-Foundry/CSF-Core/commit/78f9dea6c3525efe6a9ad845d9a4545327f99fce))

# 1.0.0 (2026-01-02)


### Bug Fixes

* add ENV_FILE support and auto-detect ORIGIN for CORS, improve config.env ([7767d84](https://github.com/CS-Foundry/CSF-Core/commit/7767d84d336d009f1c96dda578e198a6ce46c81a))
* backend as deamon service ([ca14d20](https://github.com/CS-Foundry/CSF-Core/commit/ca14d207a6254e2381b9ba7c8824aeb25dd4e5a9))
* build binarys with gh actions ([91bc5b9](https://github.com/CS-Foundry/CSF-Core/commit/91bc5b96ecaefd5d333d1bc4360d95ab84840cf7))
* Change capabilities from Vec<String> to serde_json::Value in agents entity ([524a823](https://github.com/CS-Foundry/CSF-Core/commit/524a823157777a0e7a5af3ce406c6fa4b9565906))
* create .env file for frontend build with PUBLIC_API_BASE_URL ([d6050ed](https://github.com/CS-Foundry/CSF-Core/commit/d6050ed146e01ca8b384ebe7fd8529d5f06ac599))
* docker deployment ([949d19b](https://github.com/CS-Foundry/CSF-Core/commit/949d19b36cd3ebbde77340837bba2a01866ce31c))
* error with component ([f0e86e8](https://github.com/CS-Foundry/CSF-Core/commit/f0e86e804255239b087e98e08ac5e6d3ab754982))
* improve frontend build with better error handling and npm install ([f027a20](https://github.com/CS-Foundry/CSF-Core/commit/f027a20d6d0767d25d2ab0a30287c8eb1a07a512))
* improve Rust installation and build logic ([91d1c61](https://github.com/CS-Foundry/CSF-Core/commit/91d1c619ecccd46522bc63a64a7aa80eb2e8baf3))
* installation fix ([709a676](https://github.com/CS-Foundry/CSF-Core/commit/709a676d041695c2e676da9949ef79e9cd4927e0))
* installation script ([ef58a23](https://github.com/CS-Foundry/CSF-Core/commit/ef58a23bbe86c955c4e5717d25bb7d9259fd315a))
* installation script ([03fcd72](https://github.com/CS-Foundry/CSF-Core/commit/03fcd72cb5a8491ffc412b85601e474b3602abc7))
* marketplace only docker ([263b532](https://github.com/CS-Foundry/CSF-Core/commit/263b5321e7d5483ed91b2664fec39c081ce9d274))
* redirect ([a763eee](https://github.com/CS-Foundry/CSF-Core/commit/a763eee26077b2b6164a92efd438313ec9952188))
* remove unicode characters causing bash errors ([deff5d9](https://github.com/CS-Foundry/CSF-Core/commit/deff5d90d0518f085d149685b39b0640637fdd8b))
* Resolve compilation errors in self_monitor ([efb2b3c](https://github.com/CS-Foundry/CSF-Core/commit/efb2b3cda56fca59806dc41c05d9e48b5a264599))
* semantic release workflow ([bbf0598](https://github.com/CS-Foundry/CSF-Core/commit/bbf0598ff105bfa982568907b1111be3496b178e))
* service ([23ae2f0](https://github.com/CS-Foundry/CSF-Core/commit/23ae2f08acd405f6e452bd7fc983389b84f21573))
* settings token and other things ([fa0acf9](https://github.com/CS-Foundry/CSF-Core/commit/fa0acf93f37583cd2419f3d044b13292c14b1a3d))
* support multiple CORS origins from ORIGIN env variable ([b96b530](https://github.com/CS-Foundry/CSF-Core/commit/b96b530796cdd5b6a5f76aaca1c0020d5963ea0c))
* Update self_monitor to use Json type from sea_orm ([f53b567](https://github.com/CS-Foundry/CSF-Core/commit/f53b567c2526074af10474665feb34d646487cd1))


### Features

* 2FA ([c180085](https://github.com/CS-Foundry/CSF-Core/commit/c1800850a8b2a2b02160125d8b9ebdd88e9771a6))
* Add Azure-style marketplace with Docker resource management ([dda38e3](https://github.com/CS-Foundry/CSF-Core/commit/dda38e37a8dedaa3ed5719bde646f768d865966a))
* add build-from-source support for development installations ([1e8a1a8](https://github.com/CS-Foundry/CSF-Core/commit/1e8a1a820d53601b5331a1f2f41218f214ab6b3f))
* add dev install helper script ([53e7314](https://github.com/CS-Foundry/CSF-Core/commit/53e73148b6e2e2067c5ac8c5b6c01ac5cd63a3af))
* Add full Docker container integration with auto-creation and control ([b00e239](https://github.com/CS-Foundry/CSF-Core/commit/b00e2396a9a47cb928eef43f2344e9d0f541f4b0))
* Add self-monitoring service for automatic local agent data collection ([1e42c59](https://github.com/CS-Foundry/CSF-Core/commit/1e42c59d8d0a3dd0c9b0aa2786b1416164ce4273))
* added LICENSE.txt ([82b587a](https://github.com/CS-Foundry/CSF-Core/commit/82b587ac1a66edce7b4877de3a311ae24b5d2085))
* added organization managment ([cd8dd46](https://github.com/CS-Foundry/CSF-Core/commit/cd8dd46d051bd3af54aacdc37d1ea56514339f56))
* added postgres as db and default admin user ([77ca3b9](https://github.com/CS-Foundry/CSF-Core/commit/77ca3b9051ba96f182bfb0780935e1f5a82b8574))
* agent and physical server managment ([94919c6](https://github.com/CS-Foundry/CSF-Core/commit/94919c69405016d06c581177584846941712f78d))
* agent and server architecture ([e9c6877](https://github.com/CS-Foundry/CSF-Core/commit/e9c6877d8348388846d2e6ec36232b5d56a39946))
* allow PUBLIC_API_BASE_URL to be configured via environment variable ([9c355a7](https://github.com/CS-Foundry/CSF-Core/commit/9c355a71934ab9d3f0e3c607f65c46fd82bd5d8b))
* auto-install build-essential, separate dev/prod installation logic ([9f933e6](https://github.com/CS-Foundry/CSF-Core/commit/9f933e65092a11c3e3926aaf368cd72c3d524b81))
* auto-install Rust/Cargo when building from source ([156d548](https://github.com/CS-Foundry/CSF-Core/commit/156d54808ffebe402a7292a14e2d1cf46ec04c74))
* automatically open firewall port 8000 for external access ([d2e7b10](https://github.com/CS-Foundry/CSF-Core/commit/d2e7b1052720ced1e88fe509430aa25a0cf14175))
* docker logs and docker exec ([0062cf8](https://github.com/CS-Foundry/CSF-Core/commit/0062cf844cf6869c502b45fbcbd0b56213482f0d))
* fix semantic release and binary builds ([ef6d085](https://github.com/CS-Foundry/CSF-Core/commit/ef6d085618ca36fa3d2db789805faffa07a0d203))
* installer scripts ([8440210](https://github.com/CS-Foundry/CSF-Core/commit/8440210ee6cc1cab24b0acd3988e343557918ebd))
* new ressource group managment ([5b83cb7](https://github.com/CS-Foundry/CSF-Core/commit/5b83cb7fd37bf4ca632944cf35d2fc19b519153e))

# [0.2.0](https://github.com/CS-Foundry/CSF-Core/compare/v0.1.0...v0.2.0) (2025-12-27)


### Bug Fixes

* add ENV_FILE support and auto-detect ORIGIN for CORS, improve config.env ([7767d84](https://github.com/CS-Foundry/CSF-Core/commit/7767d84d336d009f1c96dda578e198a6ce46c81a))
* backend as deamon service ([ca14d20](https://github.com/CS-Foundry/CSF-Core/commit/ca14d207a6254e2381b9ba7c8824aeb25dd4e5a9))
* create .env file for frontend build with PUBLIC_API_BASE_URL ([d6050ed](https://github.com/CS-Foundry/CSF-Core/commit/d6050ed146e01ca8b384ebe7fd8529d5f06ac599))
* improve frontend build with better error handling and npm install ([f027a20](https://github.com/CS-Foundry/CSF-Core/commit/f027a20d6d0767d25d2ab0a30287c8eb1a07a512))
* improve Rust installation and build logic ([91d1c61](https://github.com/CS-Foundry/CSF-Core/commit/91d1c619ecccd46522bc63a64a7aa80eb2e8baf3))
* installation fix ([709a676](https://github.com/CS-Foundry/CSF-Core/commit/709a676d041695c2e676da9949ef79e9cd4927e0))
* installation script ([ef58a23](https://github.com/CS-Foundry/CSF-Core/commit/ef58a23bbe86c955c4e5717d25bb7d9259fd315a))
* installation script ([03fcd72](https://github.com/CS-Foundry/CSF-Core/commit/03fcd72cb5a8491ffc412b85601e474b3602abc7))
* remove unicode characters causing bash errors ([deff5d9](https://github.com/CS-Foundry/CSF-Core/commit/deff5d90d0518f085d149685b39b0640637fdd8b))
* service ([23ae2f0](https://github.com/CS-Foundry/CSF-Core/commit/23ae2f08acd405f6e452bd7fc983389b84f21573))
* support multiple CORS origins from ORIGIN env variable ([b96b530](https://github.com/CS-Foundry/CSF-Core/commit/b96b530796cdd5b6a5f76aaca1c0020d5963ea0c))


### Features

* add build-from-source support for development installations ([1e8a1a8](https://github.com/CS-Foundry/CSF-Core/commit/1e8a1a820d53601b5331a1f2f41218f214ab6b3f))
* add dev install helper script ([53e7314](https://github.com/CS-Foundry/CSF-Core/commit/53e73148b6e2e2067c5ac8c5b6c01ac5cd63a3af))
* agent and physical server managment ([94919c6](https://github.com/CS-Foundry/CSF-Core/commit/94919c69405016d06c581177584846941712f78d))
* agent and server architecture ([e9c6877](https://github.com/CS-Foundry/CSF-Core/commit/e9c6877d8348388846d2e6ec36232b5d56a39946))
* allow PUBLIC_API_BASE_URL to be configured via environment variable ([9c355a7](https://github.com/CS-Foundry/CSF-Core/commit/9c355a71934ab9d3f0e3c607f65c46fd82bd5d8b))
* auto-install build-essential, separate dev/prod installation logic ([9f933e6](https://github.com/CS-Foundry/CSF-Core/commit/9f933e65092a11c3e3926aaf368cd72c3d524b81))
* auto-install Rust/Cargo when building from source ([156d548](https://github.com/CS-Foundry/CSF-Core/commit/156d54808ffebe402a7292a14e2d1cf46ec04c74))
* automatically open firewall port 8000 for external access ([d2e7b10](https://github.com/CS-Foundry/CSF-Core/commit/d2e7b1052720ced1e88fe509430aa25a0cf14175))
* installer scripts ([8440210](https://github.com/CS-Foundry/CSF-Core/commit/8440210ee6cc1cab24b0acd3988e343557918ebd))

# [0.1.0](https://github.com/CS-Foundry/CSF-Core/compare/v0.0.1...v0.1.0) (2025-12-15)


### Bug Fixes

* error with component ([f0e86e8](https://github.com/CS-Foundry/CSF-Core/commit/f0e86e804255239b087e98e08ac5e6d3ab754982))
* redirect ([a763eee](https://github.com/CS-Foundry/CSF-Core/commit/a763eee26077b2b6164a92efd438313ec9952188))
* semantic release workflow ([bbf0598](https://github.com/CS-Foundry/CSF-Core/commit/bbf0598ff105bfa982568907b1111be3496b178e))
* settings token and other things ([fa0acf9](https://github.com/CS-Foundry/CSF-Core/commit/fa0acf93f37583cd2419f3d044b13292c14b1a3d))


### Features

* 2FA ([c180085](https://github.com/CS-Foundry/CSF-Core/commit/c1800850a8b2a2b02160125d8b9ebdd88e9771a6))
* added organization managment ([cd8dd46](https://github.com/CS-Foundry/CSF-Core/commit/cd8dd46d051bd3af54aacdc37d1ea56514339f56))
