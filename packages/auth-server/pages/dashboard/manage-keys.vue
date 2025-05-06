<!--
 Copyright 2025 cbe

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->
<template>
  <div>
    <layout-header>
      <template #default>
        Keys
      </template>
    </layout-header>

    <div
      class="border rounded-3xl divide-y bg-white border-neutral-200 divide-neutral-200 dark:bg-neutral-950 dark:border-neutral-900 dark:divide-neutral-900"
    >
      <passkeys />
      <nuxt-child name="passkeys" />
      <nuxt-child name="sessions" />
      <sessions />
      Key Dashboards go above
    </div>

    <CommonAlert
      v-if="defaultChain.id === zksyncInMemoryNode.id"
      class="mt-4"
    >
      <template #icon>
        <InformationCircleIcon aria-hidden="true" />
      </template>
      <template #default>
        <p class="text-sm">
          Timestamps on {{ zksyncInMemoryNode.name }} start from 0 and incremented by 1 with each block. Therefore session time isn't accurate.
        </p>
      </template>
    </CommonAlert>
  </div>
</template>

<script setup lang="ts">
import { InformationCircleIcon } from "@heroicons/vue/20/solid";
import { zksyncInMemoryNode } from "viem/chains";

const { defaultChain } = useClientStore();
</script>
