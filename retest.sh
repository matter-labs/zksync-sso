#!/bin/bash
# Copyright 2025 cbe
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
# 
#     https://www.apache.org/licenses/LICENSE-2.0
# 
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
set -ex

pnpm run --dir packages/contracts build 
pnpm run --dir packages/contracts deploy --file ../auth-server/stores/local-node.json
pnpm nx build auth-server
pnpm nx e2e demo-app

