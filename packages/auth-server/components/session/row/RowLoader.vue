<template>
  <div class="session-row">
    <div class="session-id-container">
      <div>
        <CommonContentLoader :length="10" />
      </div>
      <CommonContentLoader
        class="session-created-time-ago"
        :length="20"
      />
    </div>
    <div class="session-expiry-container">
      <CommonContentLoader :length="30" />
      <CommonContentLoader class="block session-row-line" />
    </div>
    <div class="session-spend-limit-container">
      <CommonContentLoader :length="30" />
      <CommonContentLoader class="block session-row-line" />
    </div>
    <div class="session-buttons-container">
      <div class="rounded-full w-[43px] h-[43px] overflow-hidden">
        <CommonContentLoader class="block h-full w-full" />
      </div>
    </div>
  </div>
</template>
