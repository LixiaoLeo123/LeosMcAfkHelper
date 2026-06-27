// Ready-made behavior snippets the user can insert into the OnUser* hooks.
// These are *bodies* meant to be pasted inside OnUserUpdate / OnUserJoin / OnUserChat.

export interface Snippet {
  label: string;
  description: string;
  hook: "OnUserUpdate" | "OnUserJoin" | "OnUserChat";
  body: string;
}

export const snippets: Snippet[] = [
  {
    label: "Attack nearby mobs",
    description: "Each tick, find the nearest mob within 4 blocks and attack it.",
    hook: "OnUserUpdate",
    body: `if (!ClientIsMoving())
{
    var target = Game.FindNearestEntity(typeFilter: "mob", radius: 4.0);
    if (target.Ok && target.Data.Entity != null)
    {
        LookAtLocation(target.Data.Entity.Location);
        InteractEntity(target.Data.Entity.ID, InteractType.Attack);
    }
}`,
  },
  {
    label: "Pick up nearby items",
    description: "Walk to and pick up dropped items within 16 blocks (non-blocking).",
    hook: "OnUserUpdate",
    body: `if (!ClientIsMoving())
{
    Game.PickupItemsAsync("minecraft:item", radius: 16, maxItems: 10);
}`,
  },
  {
    label: "Eat when hungry",
    description: "If the food bar is low, switch to a food item and use it.",
    hook: "OnUserUpdate",
    body: `// Requires a food item in the hotbar. Adjust the item type as needed.
var sel = Game.SelectHotbarItem("minecraft:cooked_beef");
if (sel.Ok) UseItemInHand();`,
  },
  {
    label: "Wait loop (1s pause)",
    description: "A DateTime-deadline wait pattern that doesn't block the update loop.",
    hook: "OnUserUpdate",
    body: `private DateTime _next = DateTime.MinValue;
// inside OnUserUpdate:
if (DateTime.Now < _next) return;
// ...do work here...
_next = DateTime.Now.AddSeconds(1);`,
  },
  {
    label: "Move to coordinates",
    description: "Walk to a fixed location once after joining.",
    hook: "OnUserJoin",
    body: `MoveToLocation(new Location(0, 64, 0));`,
  },
  {
    label: "Respond to chat command",
    description: "When someone whispers a command, reply with a message.",
    hook: "OnUserChat",
    body: `string message = "", sender = "";
if (IsPrivateMessage(text, ref message, ref sender) && message == "!hello")
{
    SendPrivateMessage(sender, "Hi there!");
}`,
  },
];
