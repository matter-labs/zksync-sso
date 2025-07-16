<template>
  <h1>Registering your google account...</h1>
</template>

<script setup lang="ts">
if (window.opener) {
  const hashString = window.location.hash;
  window.opener.postMessage({ content: {
    type: "jwt",
    hashString,
  } });
  window.close();
}
</script>
