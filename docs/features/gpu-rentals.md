# GPU rentals

## The pain

Renting model-serving hardware exposes users to confusing hourly pricing,
unbounded spend, long setup stages, and cleanup mistakes that can continue
billing after an endpoint disappears. Corbanu Terminal makes those states and
limits explicit.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Compute: Vast.ai and RunPod rental workflows with price, spend, duration, readiness, stop, and termination controls.” |

## Before renting

The user needs a funded Vast.ai or RunPod account and the corresponding API key.
Adding a key authorizes the rental workflow; it does not add marketplace credit.

Run:

```text
/gpu
```

Use the masked credential flow when the integrated rental workflow does not
already have its canonical credential.

## Rental flow

1. Select Vast.ai or RunPod.
2. Set a maximum hourly USD price.
3. Set a maximum total USD spend.
4. Choose a duration in whole minutes.
5. Review compatible offers.
6. Confirm the final billable selection.
7. Wait through hardware checks, runtime setup, model download, artifact
   verification, loading, and endpoint qualification.
8. Select the rented model through `/model` only after the rental reports
   **READY**.

Setup and model loading consume rental time.

## Monitor and clean up

Use `/gpu` or `/gpu status` as the authoritative cross-process view of
active or potentially billable rentals, readiness, and estimated spend.

| Action | Result |
| --- | --- |
| **Stop serving** | Removes the endpoint from model selection; provider billing continues |
| **Terminate rental** | Requests provider cleanup; billing is resolved only after provider-confirmed termination |

Repeatable commands are available when the rental identifier is known:

```text
/gpu stop <id>
/gpu terminate <id>
```

Exiting Corbanu Terminal is not a substitute for terminating a rental.
