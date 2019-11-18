from django import forms
from vaalikoppi.models import Voting


class VotingForm(forms.ModelForm):
    class Meta:
        model = Voting
        fields = ["voting_name", "voting_description", "max_votes"]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
        }


class TransferableVotingForm(forms.ModelForm):
    class Meta:
        model = TransferableVoting
        fields = ["voting_name", "voting_description", "max_votes"]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
        }
