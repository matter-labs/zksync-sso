<template>
  <Dialog.Root
    :open="open"
  >
    <Dialog.Trigger
      :as-child="true"
      @click="open = true"
    >
      <slot name="trigger">
        <ZkButton>Dialog</ZkButton>
      </slot>
    </Dialog.Trigger>
    <Dialog.Portal>
      <Dialog.Overlay
        class="data-[state=open]:animate-overlayShow fixed inset-0 z-30 bg-neutral-900/70"
        @click="closeModal"
      />
      <Dialog.Content
        :class="twMerge(['data-[state=open]:animate-contentShow fixed top-[50%] left-[50%] max-h-[85vh] w-[90vw] max-w-[350px] translate-x-[-50%] translate-y-[-50%] flex flex-col rounded-zk bg-white focus:outline-none z-[100] p-6 dark:bg-neutral-950 border border-transparent dark:border-neutral-900 dark:text-neutral-100', contentClass])"
      >
        <Dialog.Title class="text-lg flex items-center mb-8">
          <span class="flex-auto">{{ title }}</span>
          <Dialog.Close
            class="inline-flex appearance-none items-center justify-center focus:outline-none focus:ring-1 rounded-full"
            aria-label="Close"
            @click="closeModal"
          >
            <ZkIcon icon="close" />
          </Dialog.Close>
        </Dialog.Title>
        <Dialog.Description :class="twMerge(['mb-10 text-lg text-center', descriptionClass])">
          <slot />
        </Dialog.Description>

        <div :class="twMerge(['flex flex-col gap-2 justify-end flex-1', closeClass])">
          <Dialog.Close as-child>
            <slot name="submit">
              <ZkButton
                type="primary"
                class="w-full"
              >
                Confirm
              </ZkButton>
            </slot>
          </Dialog.Close>
          <Dialog.Close as-child>
            <slot name="cancel">
              <ZkButton
                type="secondary"
                class="w-full"
              >
                Cancel
              </ZkButton>
            </slot>
          </Dialog.Close>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
</template>

<script setup lang="ts">
import { Dialog } from "radix-vue/namespaced";
import { twMerge } from "tailwind-merge";

const open = ref(false);

const emit = defineEmits<{
  (e: "close"): void;
}>();

const closeModal = () => {
  emit("close");
  open.value = false;
};

defineProps<{
  title: string;
  contentClass?: string;
  descriptionClass?: string;
  closeClass?: string;
}>();

defineExpose({
  close: closeModal,
});
</script>
