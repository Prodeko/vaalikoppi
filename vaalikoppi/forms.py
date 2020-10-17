from django import forms

from vaalikoppi.models import Voting, VotingTransferable


class VotingForm(forms.ModelForm):
    class Meta:
        model = Voting
        fields = ["voting_name", "voting_description", "max_votes"]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
        }


class VotingTransferableForm(forms.ModelForm):
    class Meta:
        model = VotingTransferable
        fields = ["voting_name", "voting_description", "max_votes"]
        labels = {
            "voting_name": "Äänestyksen nimi",
            "voting_description": "Äänestyksen kuvaus",
            "max_votes": "Ääniä käytössä",
        }
