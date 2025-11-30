<script lang="ts">
  import { Badge } from "$lib/components/ui/badge/index.js";

  type Props = {
    value: string;
    variant?: "default" | "secondary" | "outline" | "destructive";
  };

  let { value, variant = "outline" }: Props = $props();
</script>

<Badge {variant}>{value}</Badge>
