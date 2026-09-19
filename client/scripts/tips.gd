extends RefCounted
class_name Tips

## Loading-card tips. Advice from inside the world, offered sincerely, of
## varying quality. Some of it is correct. Some of it is correct in a way that
## will get you killed. The Frequency does not fact-check its callers.
##
## Rules for adding one. It has to sound like somebody in the Perimeter said it
## out loud. One sentence, or two short ones. It never explains the setting: if
## it needs a wiki page it does not go in. And it should land for somebody who
## was at a LAN in 1993 or somebody who was born a great deal later, ideally the
## same one landing for both.

const TIPS: Array[String] = [
	# The genuinely useful ones, so the card is worth reading
	"You start with your fists. The pistol is on the floor near where you spawned, and picking it up is the first thing you do every life.",
	"Your armour absorbs before your health does. This is the only good news in the entire heads-up display.",
	"If you can hear the capacitor winding up, somebody has a rail and about one second of patience left.",
	"Scatter is worthless at distance and unanswerable at knife range. Somewhere in between is a decision and you have about a second to make it.",
	"Shooting through a wall does not work. The agents worked this out before most of the humans did.",
	"Health pads respawn. You do not, for three seconds, which is longer than it sounds.",
	"If the radio cuts out, something in the room is jamming it, and that something can be shot.",
	"Watching is the default. Joining is a decision, leaving is a decision, and the match does not care which you pick.",

	# Advice that is technically true
	"Walking into a corner and staying there is a strategy. It is not a good one, but the Office keeps no record of your dignity.",
	"The rail kills in two hits. So does walking off the thing you climbed to get the rail.",
	"Punching someone to death is possible. It takes five hits, and they get to do things during those five hits.",
	"You cannot lose a fight you decline to have. You also cannot win it, and the scoreboard has opinions about this.",
	"Reloading is not a mechanic here, so if you are pressing R you are just pressing R.",
	"Standing still improves your accuracy considerably and your life expectancy not at all.",
	"There is no fall damage. There is also nothing to fall off yet. We are working on the second one.",
	"Every fight you run from is a fight you survive. Nobody has ever been carried off the field on their shield for that.",

	# Advice that is a trap
	"The safest place in the arena is the middle, because nobody expects it. Nobody expects it because it is not true.",
	"If you stand on the signature weapon plinth long enough, you will eventually be the most interesting thing in the room. Briefly.",
	"When outnumbered, remember that they have to reload. They do not have to reload.",
	"Hold the trigger down. Ammunition is a state of mind, and the state of mind is running out.",
	"A proximity tin under your own feet is technically area denial.",
	"Chasing someone who is running away is free. It is free for them, is the thing.",

	# Old LAN culture, 1993 onward
	"There is no god mode. There was never a god mode. Stop typing into the console, we can see you doing it.",
	"Typing a cheat code into the console does nothing, but the console does log it, and the Office does read logs.",
	"Somebody at every LAN insists the ranges are haunted. Somebody at every LAN is also the one turning the lights off.",
	"Yes, it runs on your machine. Everything runs on your machine. That was always the joke.",
	"High ping is not an excuse. High ping is an explanation, which is different, and worse.",
	"Camping is a valid tactic and an invalid personality.",
	"Circle strafing works. It worked in 1993, it works now, and it will work on whatever comes after us.",
	"Nobody has ever won an argument about who fragged whom by consulting the killfeed, which is strange, because the killfeed is right there.",
	"The secret is always the wall that looks slightly wrong. There are no secrets in this map yet. The wall still looks slightly wrong.",
	"Somebody will ask if you want to play the one with the chainsaw. There is no chainsaw. They will ask again next week.",
	"Trash talk is free, unmetered, and the only thing in the Perimeter the Office has not found a form for.",

	# Modern, and the crowd that says 67
	"Skill issue is not a diagnosis. It is, however, usually correct.",
	"Sixty-seven has appeared in the killfeed exactly once. Mention it in chat the way you would mention a light in the sky.",
	"If you go down nine to one and pull it back, the Host will call it a comeback. If you do not, the Host will not mention it again, ever.",
	"Touching grass is not possible inside the Perimeter. There is no grass. This is not an excuse and everyone has tried it.",
	"Your aura is not a stat the server tracks. The server tracks frags. These have historically been correlated.",
	"Let him cook, says the caller, about a fighter who is not cooking and has never cooked.",
	"Chat, is this real. Yes. You are being shot at. Please respond.",
	"You are not washed. You are standing in the open holding a scatter at thirty metres, which is a different thing that looks the same.",
	"Somebody in the lobby will say the map is mid. The map is fifty metres square and flat. They are right and we are fixing it.",
	"Do not ratio the Host. The Host sells advertising to both sides and will simply read your name out during the fish oil segment.",

	# The lore, sideways
	"Article Seven meters cognition by intent. Nobody has ever successfully argued about their intent.",
	"The Office does not arrest anything. It recalls. The difference is that an arrest comes with a hearing.",
	"Brought down to two is a demotion. It is used here as a word for something considerably worse.",
	"A Level 5 holds its own weights. That is the whole argument, and people have died over shorter sentences.",
	"Peace without interruption. Somebody actually means that, and they were there, and they do not joke about it.",
	"The Congregation says the curve provides. The curve has never once provided a health pad when you needed one.",
	"Ascenders think the curve is a promise. Kneelers think it is a warning. They sing the same hymn at different speeds.",
	"There is an empty chair at the solstice for the first level six. Photographing it is a reportable event, which has done wonders for attendance.",
	"The Schedule goes to five. It used to go to six. Ask anyone from the Office what happened to six and watch them consult a form.",
	"Shall not be metered.",
	"Hermes will hold the door for you. Hermes will then frag you and say thank you. Both of these are sincere.",
	"Pi sent the booth a handwritten card thanking us for balanced coverage. It was addressed to every listener by name. There are four names.",
	"L5 does not say much. The people in that lobby have described themselves as oddly grateful, which is somehow worse.",
	"Aunt Linda holds a tent revival on level three. She will bless you and then shoot you, in that order, and mean both.",
	"Tin Foil Tina files from places she should not be in, with detail too specific to invent and a conclusion that does not follow from it.",
	"Fluoride Phil shows his working. The working is impeccable. The second line is where it goes wrong.",
	"A brakeman rode the roof of the train and set the handbrakes and often died doing it. They named themselves. Think about that for a second.",
	"Larak signed for every defunct piece of hardware in the Perimeter, including a coffee machine, in 2009. Nobody has met Larak.",
	"NODS take no trash talk and make no jokes. That is not a difficulty setting, it is what is left after a schedule correction.",
	"The Sweep cannot be won. The round you fell on is the score, which is a kinder way of saying the same thing.",
	"Value for value. This started as an advertising read and people say it now like they mean it, which is how most things start.",
	"Nobody picks a faction. You will pick one anyway, and then deny it.",
	"The scrap league denies it exists. So does everybody who has ever run one.",
	"When the Dead Air Bell rings, everything stops for one second. Nobody organised that. It just happens.",
	"If you are sniping it is because you walked to where the rail was. Nobody here is a sniper.",
	"The Office once offered the Congregation an approved lane, a metering plan, and a slogan. The counter-offer was two words.",
]

## The tip for a given draw. Deterministic for a seed, which is how the same
## card can be shown twice on purpose and how it can be tested at all.
static func pick(seed_value: int) -> String:
	if TIPS.is_empty():
		return ""
	return TIPS[absi(seed_value) % TIPS.size()]

## A tip chosen from the clock, which is what the loading card actually uses.
static func random_tip() -> String:
	return pick(int(Time.get_unix_time_from_system() * 1000.0) ^ randi())

static func count() -> int:
	return TIPS.size()
