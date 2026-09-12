# 24. Recover the affected account, not the current model

After an OpenAI connected-app authentication failure, open `/providers`. OpenAI should need attention while the working Claude provider remains configured. Select OpenAI, press `r`, and choose sign-in. Cancel once, reopen, then finish sign-in.

Pass: cancellation changes nothing; successful sign-in refreshes status and the app connection without restarting or changing provider/model.

