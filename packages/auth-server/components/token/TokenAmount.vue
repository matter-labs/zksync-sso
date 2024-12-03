<template>
  <CommonButtonLineWithImg :as="as">
    <template #image>
      <TokenImage
        :symbol="symbol"
        :address="address"
        :icon-url="iconUrl"
      />
    </template>
    <template #default>
      <CommonButtonLineBodyInfo class="text-left">
        <template #label>
          <div class="truncate">
            <span
              class="font-medium"
              :title="unformattedAmount"
            >{{ formattedAmount }}</span>
            <span
              v-if="tokenPrice"
              class="text-neutral-500 text-sm"
            >&nbsp;(~{{ tokenPrice }})</span>
          </div>
        </template>
        <template
          v-if="symbol || name"
          #underline
        >
          <div class="truncate">
            {{ symbol || name }}
          </div>
        </template>
      </CommonButtonLineBodyInfo>
    </template>
    <template #right>
      <slot name="right" />
    </template>
  </CommonButtonLineWithImg>
</template>

<script lang="ts" setup>
import { formatUnits } from "viem";

const props = defineProps({
  as: {
    type: [String, Object] as PropType<string | Component>,
  },
  symbol: {
    type: String,
    required: true,
  },
  name: {
    type: String,
  },
  address: {
    type: String,
    required: true,
  },
  decimals: {
    type: Number,
    required: true,
  },
  iconUrl: {
    type: String,
  },
  amount: {
    type: String as PropType<string | "unlimited">,
    required: true,
  },
  price: {
    type: [String, Number] as PropType<number | undefined>,
  },
});

const isUnlimitedAmount = computed(() => props.amount === "unlimited");
const unformattedAmount = computed(() => {
  if (isUnlimitedAmount.value) return undefined;
  return formatUnits(BigInt(props.amount), props.decimals);
});
const formattedAmount = computed(() => {
  if (isUnlimitedAmount.value) return "Unlimited";
  return formatAmount(BigInt(props.amount), props.decimals);
});
const tokenPrice = computed(() => {
  if (!props.price || isUnlimitedAmount.value) return;
  return formatTokenPrice(BigInt(props.amount), props.decimals, props.price);
});
</script>

<style lang="scss" scoped></style>
